use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct LessonEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub lesson_type: String,
    pub instructor: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct ExpenseEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub description: String,
    pub amount: f64,
    pub category: String,
}

#[derive(Debug, Default)]
pub struct DriversLicenseStats {
    pub total_lessons: usize,
    pub lessons_this_month: usize,
    pub by_lesson_type: Vec<(String, usize)>,
    pub total_spent: f64,
    pub spent_this_month: f64,
    pub by_category: Vec<(String, f64)>,
}

#[derive(Debug)]
pub struct DriversLicenseTracker {
    conn: Option<Connection>,
}

impl DriversLicenseTracker {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).ok().and_then(|conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS dl_lessons (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    lesson_type TEXT NOT NULL,
                    instructor TEXT NOT NULL,
                    notes TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS dl_expenses (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    description TEXT NOT NULL,
                    amount REAL NOT NULL,
                    category TEXT NOT NULL
                );",
            )
            .ok()?;
            Some(conn)
        });
        Self { conn }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn add_lesson(&mut self, date: NaiveDate, lesson_type: &str, instructor: &str, notes: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "INSERT INTO dl_lessons (date, lesson_type, instructor, notes) VALUES (?1, ?2, ?3, ?4)",
            params![date.format("%Y-%m-%d").to_string(), lesson_type, instructor, notes],
        )
        .unwrap();
    }

    pub fn delete_lesson(&mut self, id: i64) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute("DELETE FROM dl_lessons WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn update_lesson(&mut self, id: i64, date: NaiveDate, lesson_type: &str, instructor: &str, notes: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "UPDATE dl_lessons SET date=?1, lesson_type=?2, instructor=?3, notes=?4 WHERE id=?5",
            params![date.format("%Y-%m-%d").to_string(), lesson_type, instructor, notes, id],
        )
        .unwrap();
    }

    pub fn load_all_lessons(&self) -> Vec<LessonEntry> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT id, date, lesson_type, instructor, notes FROM dl_lessons ORDER BY date DESC")
            .unwrap();

        stmt.query_map([], |row| {
            Ok(LessonEntry {
                id: row.get(0)?,
                date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                lesson_type: row.get(2)?,
                instructor: row.get(3)?,
                notes: row.get(4)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
    }

    pub fn unique_lesson_types(&self) -> Vec<String> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT DISTINCT lesson_type FROM dl_lessons ORDER BY lesson_type")
            .unwrap();

        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn add_expense(&mut self, date: NaiveDate, description: &str, amount: f64, category: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "INSERT INTO dl_expenses (date, description, amount, category) VALUES (?1, ?2, ?3, ?4)",
            params![date.format("%Y-%m-%d").to_string(), description, amount, category],
        )
        .unwrap();
    }

    pub fn delete_expense(&mut self, id: i64) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute("DELETE FROM dl_expenses WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn update_expense(&mut self, id: i64, date: NaiveDate, description: &str, amount: f64, category: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "UPDATE dl_expenses SET date=?1, description=?2, amount=?3, category=?4 WHERE id=?5",
            params![date.format("%Y-%m-%d").to_string(), description, amount, category, id],
        )
        .unwrap();
    }

    pub fn load_all_expenses(&self) -> Vec<ExpenseEntry> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT id, date, description, amount, category FROM dl_expenses ORDER BY date DESC")
            .unwrap();

        stmt.query_map([], |row| {
            Ok(ExpenseEntry {
                id: row.get(0)?,
                date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                description: row.get(2)?,
                amount: row.get(3)?,
                category: row.get(4)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
    }

    pub fn unique_expense_categories(&self) -> Vec<String> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT DISTINCT category FROM dl_expenses ORDER BY category")
            .unwrap();

        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn export_lessons_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.conn.is_none() {
            return Err("No database connection".into());
        }
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "id,date,lesson_type,instructor,notes")?;
        for e in self.load_all_lessons() {
            writeln!(f, "{},{},{},{},{}", e.id, e.date, e.lesson_type, e.instructor, e.notes)?;
        }
        Ok(())
    }

    pub fn export_expenses_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.conn.is_none() {
            return Err("No database connection".into());
        }
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "id,date,description,amount,category")?;
        for e in self.load_all_expenses() {
            writeln!(f, "{},{},{},{:.2},{}", e.id, e.date, e.description, e.amount, e.category)?;
        }
        Ok(())
    }

    pub fn stats(&self) -> DriversLicenseStats {
        let lessons = self.load_all_lessons();
        let expenses = self.load_all_expenses();
        let today = chrono::Local::now().date_naive();

        let lessons_this_month = lessons
            .iter()
            .filter(|e| e.date.year() == today.year() && e.date.month() == today.month())
            .count();

        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for l in &lessons {
            *type_counts.entry(&l.lesson_type).or_default() += 1;
        }
        let mut by_lesson_type: Vec<(String, usize)> =
            type_counts.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        by_lesson_type.sort_by(|a, b| b.1.cmp(&a.1));

        let total_spent: f64 = expenses.iter().map(|e| e.amount).sum();
        let spent_this_month: f64 = expenses
            .iter()
            .filter(|e| e.date.year() == today.year() && e.date.month() == today.month())
            .map(|e| e.amount)
            .sum();

        let mut cat_amounts: HashMap<&str, f64> = HashMap::new();
        for e in &expenses {
            *cat_amounts.entry(&e.category).or_default() += e.amount;
        }
        let mut by_category: Vec<(String, f64)> =
            cat_amounts.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
        by_category.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        DriversLicenseStats {
            total_lessons: lessons.len(),
            lessons_this_month,
            by_lesson_type,
            total_spent,
            spent_this_month,
            by_category,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{date, dl_tracker};

    // --- Lessons ---

    #[test]
    fn empty_db_has_no_lessons_or_expenses() {
        let dt = dl_tracker();
        assert!(dt.load_all_lessons().is_empty());
        assert!(dt.load_all_expenses().is_empty());
    }

    #[test]
    fn lesson_add_and_load_roundtrip() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 4, 5), "Highway", "Hans", "Good lesson");
        let lessons = dt.load_all_lessons();
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].date, date(2024, 4, 5));
        assert_eq!(lessons[0].lesson_type, "Highway");
        assert_eq!(lessons[0].instructor, "Hans");
        assert_eq!(lessons[0].notes, "Good lesson");
    }

    #[test]
    fn lesson_delete() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "City", "Anna", "");
        dt.add_lesson(date(2024, 1, 2), "Highway", "Hans", "");
        let id = dt.load_all_lessons().iter().find(|l| l.lesson_type == "City").unwrap().id;
        dt.delete_lesson(id);
        let lessons = dt.load_all_lessons();
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].lesson_type, "Highway");
    }

    #[test]
    fn lesson_update() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "City", "Old", "");
        let id = dt.load_all_lessons()[0].id;
        dt.update_lesson(id, date(2024, 6, 1), "Highway", "New", "Updated");
        let lessons = dt.load_all_lessons();
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].lesson_type, "Highway");
        assert_eq!(lessons[0].instructor, "New");
        assert_eq!(lessons[0].notes, "Updated");
        assert_eq!(lessons[0].date, date(2024, 6, 1));
    }

    #[test]
    fn lessons_ordered_date_desc() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "A", "X", "");
        dt.add_lesson(date(2024, 3, 1), "B", "X", "");
        dt.add_lesson(date(2024, 2, 1), "C", "X", "");
        let lessons = dt.load_all_lessons();
        assert_eq!(lessons[0].date, date(2024, 3, 1));
        assert_eq!(lessons[2].date, date(2024, 1, 1));
    }

    #[test]
    fn unique_lesson_types_sorted_and_deduped() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "Highway", "X", "");
        dt.add_lesson(date(2024, 1, 2), "City", "X", "");
        dt.add_lesson(date(2024, 1, 3), "Highway", "Y", "");
        assert_eq!(dt.unique_lesson_types(), vec!["City", "Highway"]);
    }

    // --- Expenses ---

    #[test]
    fn expense_add_and_load_roundtrip() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 4, 5), "Theory book", 29.99, "Materials");
        let expenses = dt.load_all_expenses();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].date, date(2024, 4, 5));
        assert_eq!(expenses[0].description, "Theory book");
        assert!((expenses[0].amount - 29.99).abs() < 0.001);
        assert_eq!(expenses[0].category, "Materials");
    }

    #[test]
    fn expense_delete() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 1, 1), "Book", 20.0, "Materials");
        dt.add_expense(date(2024, 1, 2), "Lesson fee", 50.0, "Fees");
        let id = dt.load_all_expenses().iter().find(|e| e.description == "Book").unwrap().id;
        dt.delete_expense(id);
        let expenses = dt.load_all_expenses();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].description, "Lesson fee");
    }

    #[test]
    fn expense_update() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 1, 1), "Old", 10.0, "OldCat");
        let id = dt.load_all_expenses()[0].id;
        dt.update_expense(id, date(2024, 6, 1), "New", 99.99, "NewCat");
        let expenses = dt.load_all_expenses();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].description, "New");
        assert!((expenses[0].amount - 99.99).abs() < 0.001);
        assert_eq!(expenses[0].category, "NewCat");
    }

    #[test]
    fn unique_expense_categories_sorted_and_deduped() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 1, 1), "A", 10.0, "Fees");
        dt.add_expense(date(2024, 1, 2), "B", 20.0, "Materials");
        dt.add_expense(date(2024, 1, 3), "C", 30.0, "Fees");
        assert_eq!(dt.unique_expense_categories(), vec!["Fees", "Materials"]);
    }

    // --- Stats ---

    #[test]
    fn stats_on_empty_db() {
        let s = dl_tracker().stats();
        assert_eq!(s.total_lessons, 0);
        assert_eq!(s.total_spent, 0.0);
        assert!(s.by_lesson_type.is_empty());
        assert!(s.by_category.is_empty());
    }

    #[test]
    fn stats_totals() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "City", "X", "");
        dt.add_lesson(date(2024, 1, 2), "Highway", "X", "");
        dt.add_expense(date(2024, 1, 1), "A", 30.0, "Fees");
        dt.add_expense(date(2024, 1, 2), "B", 20.5, "Materials");
        let s = dt.stats();
        assert_eq!(s.total_lessons, 2);
        assert!((s.total_spent - 50.5).abs() < 0.001);
    }

    #[test]
    fn stats_this_month() {
        let today = chrono::Local::now().date_naive();
        let mut dt = dl_tracker();
        dt.add_lesson(today, "City", "X", "");
        dt.add_lesson(today, "Highway", "X", "");
        dt.add_lesson(date(2000, 1, 1), "City", "X", "");
        dt.add_expense(today, "A", 30.0, "Fees");
        dt.add_expense(date(2000, 1, 1), "B", 10.0, "Fees");
        let s = dt.stats();
        assert_eq!(s.lessons_this_month, 2);
        assert!((s.spent_this_month - 30.0).abs() < 0.001);
    }

    #[test]
    fn stats_by_lesson_type_sorted_desc() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 1, 1), "City", "X", "");
        dt.add_lesson(date(2024, 1, 2), "Highway", "X", "");
        dt.add_lesson(date(2024, 1, 3), "Highway", "X", "");
        let s = dt.stats();
        assert_eq!(s.by_lesson_type[0], ("Highway".to_string(), 2));
        assert_eq!(s.by_lesson_type[1], ("City".to_string(), 1));
    }

    #[test]
    fn stats_by_category_sorted_desc_by_amount() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 1, 1), "A", 10.0, "Fees");
        dt.add_expense(date(2024, 1, 2), "B", 50.0, "Materials");
        dt.add_expense(date(2024, 1, 3), "C", 5.0, "Fees");
        let s = dt.stats();
        assert_eq!(s.by_category[0].0, "Materials");
        assert!((s.by_category[0].1 - 50.0).abs() < 0.001);
        assert_eq!(s.by_category[1].0, "Fees");
        assert!((s.by_category[1].1 - 15.0).abs() < 0.001);
    }

    #[test]
    fn export_lessons_csv_format() {
        let mut dt = dl_tracker();
        dt.add_lesson(date(2024, 3, 10), "Highway", "Hans", "Nice");
        let tmp = tempfile::NamedTempFile::new().unwrap();
        dt.export_lessons_csv(tmp.path().to_str().unwrap()).unwrap();
        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines[0], "id,date,lesson_type,instructor,notes");
        assert!(lines[1].contains("2024-03-10,Highway,Hans,Nice"));
    }

    #[test]
    fn export_expenses_csv_format() {
        let mut dt = dl_tracker();
        dt.add_expense(date(2024, 3, 10), "Theory book", 29.99, "Materials");
        let tmp = tempfile::NamedTempFile::new().unwrap();
        dt.export_expenses_csv(tmp.path().to_str().unwrap()).unwrap();
        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines[0], "id,date,description,amount,category");
        assert!(lines[1].contains("2024-03-10,Theory book,29.99,Materials"));
    }
}
