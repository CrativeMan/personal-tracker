#!/usr/bin/env python3
"""Seed ./work_tracker.db with randomised test data."""

import random
import sqlite3
from datetime import date, timedelta

DB_PATH = "./work_tracker.db"

STATIONS = ["Fürstenfeldbruck", "Germering", "Bergkirchen", "Mammendorf"]
SHIFTS = ["Early", "Late", "Night", "Day"]

# Weighted so some stations/shifts appear more often, making stats interesting
STATION_WEIGHTS = [3, 2, 4, 1]
SHIFT_WEIGHTS = [4, 3, 2, 1]

NUM_ENTRIES = 120
START_DATE = date.today() - timedelta(days=365)


def main():
    conn = sqlite3.connect(DB_PATH)
    conn.execute(
        """CREATE TABLE IF NOT EXISTS work_entries (
               id      INTEGER PRIMARY KEY AUTOINCREMENT,
               date    TEXT NOT NULL,
               station TEXT NOT NULL,
               shift   TEXT NOT NULL
           )"""
    )

    existing = conn.execute("SELECT COUNT(*) FROM work_entries").fetchone()[0]

    rows = []
    for _ in range(NUM_ENTRIES):
        offset = random.randint(0, 365)
        d = START_DATE + timedelta(days=offset)
        station = random.choices(STATIONS, weights=STATION_WEIGHTS)[0]
        shift = random.choices(SHIFTS, weights=SHIFT_WEIGHTS)[0]
        rows.append((d.isoformat(), station, shift))

    conn.executemany(
        "INSERT INTO work_entries (date, station, shift) VALUES (?, ?, ?)", rows
    )
    conn.commit()
    conn.close()

    print(
        f"Done. Added {NUM_ENTRIES} entries (was {existing}, now {existing + NUM_ENTRIES})."
    )


if __name__ == "__main__":
    main()
