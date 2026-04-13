use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WorkEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub station: String,
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
                station TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        Self { conn }
    }

    pub fn load_all(&self) -> Vec<WorkEntry> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, date, station FROM work_entries ORDER BY date DESC")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(WorkEntry {
                    id: row.get(0)?,
                    date: NaiveDate::parse_from_str(&row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                    station: row.get(2)?,
                })
            })
            .unwrap();

        rows.map(|r| r.unwrap()).collect()
    }

    pub fn add(&mut self, date: NaiveDate, station: &str) {
        self.conn
            .execute(
                "INSERT INTO work_entries (date, station) VALUES (?1, ?2)",
                params![date.format("%Y-%m-%d").to_string(), station],
            )
            .unwrap();
    }

    pub fn delete(&mut self, id: i64) {
        self.conn
            .execute("DELETE FROM work_entries WHERE id = ?1", params![id])
            .unwrap();
    }
}
