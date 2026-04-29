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
    conn: Connection,
}

impl DriversLicenseTracker {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("db open failed");

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
        .unwrap();

        Self { conn }
    }

    pub fn add_lesson(&mut self, date: NaiveDate, lesson_type: &str, instructor: &str, notes: &str) {
        self.conn
            .execute(
                "INSERT INTO dl_lessons (date, lesson_type, instructor, notes) VALUES (?1, ?2, ?3, ?4)",
                params![date.format("%Y-%m-%d").to_string(), lesson_type, instructor, notes],
            )
            .unwrap();
    }

    pub fn delete_lesson(&mut self, id: i64) {
        self.conn
            .execute("DELETE FROM dl_lessons WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn update_lesson(&mut self, id: i64, date: NaiveDate, lesson_type: &str, instructor: &str, notes: &str) {
        self.conn
            .execute(
                "UPDATE dl_lessons SET date=?1, lesson_type=?2, instructor=?3, notes=?4 WHERE id=?5",
                params![date.format("%Y-%m-%d").to_string(), lesson_type, instructor, notes, id],
            )
            .unwrap();
    }

    pub fn load_all_lessons(&self) -> Vec<LessonEntry> {
        let mut stmt = self
            .conn
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
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT lesson_type FROM dl_lessons ORDER BY lesson_type")
            .unwrap();

        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn add_expense(&mut self, date: NaiveDate, description: &str, amount: f64, category: &str) {
        self.conn
            .execute(
                "INSERT INTO dl_expenses (date, description, amount, category) VALUES (?1, ?2, ?3, ?4)",
                params![date.format("%Y-%m-%d").to_string(), description, amount, category],
            )
            .unwrap();
    }

    pub fn delete_expense(&mut self, id: i64) {
        self.conn
            .execute("DELETE FROM dl_expenses WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn update_expense(&mut self, id: i64, date: NaiveDate, description: &str, amount: f64, category: &str) {
        self.conn
            .execute(
                "UPDATE dl_expenses SET date=?1, description=?2, amount=?3, category=?4 WHERE id=?5",
                params![date.format("%Y-%m-%d").to_string(), description, amount, category, id],
            )
            .unwrap();
    }

    pub fn load_all_expenses(&self) -> Vec<ExpenseEntry> {
        let mut stmt = self
            .conn
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
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT category FROM dl_expenses ORDER BY category")
            .unwrap();

        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn export_lessons_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "id,date,lesson_type,instructor,notes")?;
        for e in self.load_all_lessons() {
            writeln!(f, "{},{},{},{},{}", e.id, e.date, e.lesson_type, e.instructor, e.notes)?;
        }
        Ok(())
    }

    pub fn export_expenses_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
