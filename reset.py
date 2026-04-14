#!/usr/bin/env python3
"""Reset ./work_tracker.db — deletes all rows and resets the auto-increment counter."""

import sqlite3
import sys

DB_PATH = "./work_tracker.db"


def main():
    if "--yes" not in sys.argv:
        answer = input("This will delete ALL entries. Continue? [y/N] ").strip().lower()
        if answer != "y":
            print("Aborted.")
            return

    conn = sqlite3.connect(DB_PATH)

    count = conn.execute("SELECT COUNT(*) FROM work_entries").fetchone()[0]
    conn.execute("DELETE FROM work_entries")
    conn.execute("DELETE FROM sqlite_sequence WHERE name='work_entries'")
    conn.commit()
    conn.close()

    print(f"Done. Removed {count} entries. Auto-increment counter reset.")


if __name__ == "__main__":
    main()
