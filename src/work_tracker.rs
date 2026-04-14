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
    conn: Connection,
}

impl WorkTracker {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("db open failed");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS work_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                station TEXT NOT NULL,
                shift TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        Self { conn }
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
        let mut stmt = self
            .conn
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
        self.conn
            .execute(
                "INSERT INTO work_entries (date, station, shift) VALUES (?1, ?2, ?3)",
                params![date.format("%Y-%m-%d").to_string(), station, shift],
            )
            .unwrap();
    }

    pub fn delete(&mut self, id: i64) {
        self.conn
            .execute("DELETE FROM work_entries WHERE id = ?1", params![id])
            .unwrap();
    }
}
