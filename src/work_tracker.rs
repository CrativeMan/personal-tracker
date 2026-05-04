use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WorkEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub station: String,
    pub shift: String,
}

#[derive(Debug, Default)]
pub struct WorkStats {
    pub total_shifts: usize,
    pub shifts_this_month: usize,
    pub unique_stations: usize,
    pub most_common_shift: Option<(String, usize)>,
    pub by_station: Vec<(String, usize)>, // sorted desc
    pub by_shift: Vec<(String, usize)>,   // sorted desc
}

#[derive(Debug)]
pub struct WorkTracker {
    conn: Option<Connection>,
}

impl WorkTracker {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).ok().and_then(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS work_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    station TEXT NOT NULL,
                    shift TEXT NOT NULL
                )",
                [],
            )
            .ok()?;
            Some(conn)
        });
        Self { conn }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn stats(&self) -> WorkStats {
        let entries = self.load_all();
        let today = chrono::Local::now().date_naive();

        let shifts_this_month = entries
            .iter()
            .filter(|e| e.date.year() == today.year() && e.date.month() == today.month())
            .count();

        let mut station_counts: std::collections::HashMap<&str, usize> = HashMap::new();
        let mut shift_counts: std::collections::HashMap<&str, usize> = HashMap::new();

        for e in &entries {
            *station_counts.entry(&e.station).or_default() += 1;
            *shift_counts.entry(&e.shift).or_default() += 1;
        }

        let mut by_station: Vec<(String, usize)> = station_counts
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
        by_station.sort_by(|a, b| b.1.cmp(&a.1));

        let mut by_shift: Vec<(String, usize)> = shift_counts
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
        by_shift.sort_by(|a, b| b.1.cmp(&a.1));

        let most_common_shift = by_shift.first().cloned();

        WorkStats {
            total_shifts: entries.len(),
            shifts_this_month,
            unique_stations: by_station.len(),
            most_common_shift,
            by_station,
            by_shift,
        }
    }

    pub fn load_all(&self) -> Vec<WorkEntry> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT id, date, station, shift FROM work_entries ORDER BY date DESC")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(WorkEntry {
                    id: row.get(0)?,
                    date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                    station: row.get(2)?,
                    shift: row.get(3)?,
                })
            })
            .unwrap();

        rows.map(|r| r.unwrap()).collect()
    }

    pub fn add(&mut self, date: NaiveDate, station: &str, shift: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "INSERT INTO work_entries (date, station, shift) VALUES (?1, ?2, ?3)",
            params![date.format("%Y-%m-%d").to_string(), station, shift],
        )
        .unwrap();
    }

    pub fn delete(&mut self, id: i64) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute("DELETE FROM work_entries WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn update(&mut self, id: i64, date: NaiveDate, station: &str, shift: &str) {
        let Some(conn) = &mut self.conn else { return; };
        conn.execute(
            "UPDATE work_entries SET date=?1, station=?2, shift=?3 WHERE id=?4",
            params![date.format("%Y-%m-%d").to_string(), station, shift, id],
        )
        .unwrap();
    }

    pub fn unique_stations(&self) -> Vec<String> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT DISTINCT station FROM work_entries ORDER BY station")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn unique_shifts(&self) -> Vec<String> {
        let Some(conn) = &self.conn else { return vec![]; };
        let mut stmt = conn
            .prepare("SELECT DISTINCT shift FROM work_entries ORDER BY shift")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn export_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.conn.is_none() {
            return Err("No database connection".into());
        }
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "id,date,station,shift")?;
        for e in self.load_all() {
            writeln!(f, "{},{},{},{}", e.id, e.date, e.station, e.shift)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{date, work_tracker};

    #[test]
    fn empty_db_has_no_entries() {
        assert!(work_tracker().load_all().is_empty());
    }

    #[test]
    fn add_and_load_roundtrip() {
        let mut wt = work_tracker();
        wt.add(date(2024, 3, 10), "Mitte", "Morning");
        let entries = wt.load_all();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].date, date(2024, 3, 10));
        assert_eq!(entries[0].station, "Mitte");
        assert_eq!(entries[0].shift, "Morning");
    }

    #[test]
    fn load_all_ordered_date_desc() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "A", "Morning");
        wt.add(date(2024, 3, 1), "B", "Evening");
        wt.add(date(2024, 2, 1), "C", "Night");
        let entries = wt.load_all();
        assert_eq!(entries[0].date, date(2024, 3, 1));
        assert_eq!(entries[1].date, date(2024, 2, 1));
        assert_eq!(entries[2].date, date(2024, 1, 1));
    }

    #[test]
    fn delete_removes_entry() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "A", "Morning");
        wt.add(date(2024, 1, 2), "B", "Evening");
        let id = wt.load_all().into_iter().find(|e| e.station == "A").unwrap().id;
        wt.delete(id);
        let entries = wt.load_all();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].station, "B");
    }

    #[test]
    fn update_changes_fields() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "Old", "Morning");
        let id = wt.load_all()[0].id;
        wt.update(id, date(2024, 6, 15), "New", "Night");
        let entries = wt.load_all();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].station, "New");
        assert_eq!(entries[0].shift, "Night");
        assert_eq!(entries[0].date, date(2024, 6, 15));
    }

    #[test]
    fn stats_on_empty_db() {
        let s = work_tracker().stats();
        assert_eq!(s.total_shifts, 0);
        assert_eq!(s.unique_stations, 0);
        assert!(s.most_common_shift.is_none());
    }

    #[test]
    fn stats_totals_and_uniques() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "A", "Morning");
        wt.add(date(2024, 1, 2), "B", "Morning");
        wt.add(date(2024, 1, 3), "A", "Evening");
        let s = wt.stats();
        assert_eq!(s.total_shifts, 3);
        assert_eq!(s.unique_stations, 2);
    }

    #[test]
    fn stats_this_month_only_counts_current_month() {
        let today = chrono::Local::now().date_naive();
        let mut wt = work_tracker();
        wt.add(today, "A", "Morning");
        wt.add(today, "B", "Evening");
        wt.add(date(2000, 1, 1), "C", "Night");
        let s = wt.stats();
        assert_eq!(s.shifts_this_month, 2);
        assert_eq!(s.total_shifts, 3);
    }

    #[test]
    fn stats_most_common_shift() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "A", "Morning");
        wt.add(date(2024, 1, 2), "B", "Morning");
        wt.add(date(2024, 1, 3), "A", "Evening");
        let (shift, count) = wt.stats().most_common_shift.unwrap();
        assert_eq!(shift, "Morning");
        assert_eq!(count, 2);
    }

    #[test]
    fn stats_by_station_sorted_desc() {
        let mut wt = work_tracker();
        wt.add(date(2024, 1, 1), "B", "Morning");
        wt.add(date(2024, 1, 2), "A", "Morning");
        wt.add(date(2024, 1, 3), "A", "Evening");
        let s = wt.stats();
        assert_eq!(s.by_station[0], ("A".to_string(), 2));
        assert_eq!(s.by_station[1], ("B".to_string(), 1));
    }

    #[test]
    fn export_csv_writes_header_and_rows() {
        let mut wt = work_tracker();
        wt.add(date(2024, 3, 10), "Mitte", "Morning");
        let tmp = tempfile::NamedTempFile::new().unwrap();
        wt.export_csv(tmp.path().to_str().unwrap()).unwrap();
        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines[0], "id,date,station,shift");
        assert!(lines[1].contains("2024-03-10,Mitte,Morning"));
    }
}
