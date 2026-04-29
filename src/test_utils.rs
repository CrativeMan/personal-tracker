use chrono::NaiveDate;

use crate::{
    drivers_license_tracker::DriversLicenseTracker,
    work_tracker::WorkTracker,
};

/// Construct a NaiveDate without the verbosity of from_ymd_opt.
pub fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// WorkTracker backed by an in-memory SQLite database.
pub fn work_tracker() -> WorkTracker {
    WorkTracker::new(":memory:")
}

/// DriversLicenseTracker backed by an in-memory SQLite database.
pub fn dl_tracker() -> DriversLicenseTracker {
    DriversLicenseTracker::new(":memory:")
}
