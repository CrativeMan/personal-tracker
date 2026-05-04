use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

fn settings_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tracker");
    std::fs::create_dir_all(&dir).ok();
    dir.join("settings.json")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DateFormat {
    #[default]
    Iso,
    European,
    American,
}

impl DateFormat {
    pub fn format(&self, date: NaiveDate) -> String {
        match self {
            DateFormat::Iso => date.format("%Y-%m-%d").to_string(),
            DateFormat::European => date.format("%d.%m.%Y").to_string(),
            DateFormat::American => date.format("%m/%d/%Y").to_string(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            DateFormat::Iso => "YYYY-MM-DD",
            DateFormat::European => "DD.MM.YYYY",
            DateFormat::American => "MM/DD/YYYY",
        }
    }
}

pub const ACCENT_PALETTE: [[u8; 3]; 5] = [
    [0x1D, 0x9E, 0x75], // teal (default)
    [0x3B, 0x82, 0xF6], // blue
    [0x8B, 0x5C, 0xF6], // purple
    [0xF5, 0x9E, 0x0B], // amber
    [0xF4, 0x3F, 0x5E], // rose
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub pixels_per_point: f32,
    pub date_format: DateFormat,
    pub accent_color: [u8; 3],
    pub compact_mode: bool,
    pub data_dir: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            pixels_per_point: 1.0,
            date_format: DateFormat::Iso,
            accent_color: ACCENT_PALETTE[0],
            compact_mode: false,
            data_dir: ".".to_string(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(settings_path(), json);
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        ctx.set_pixels_per_point(self.pixels_per_point);
    }

    pub fn work_db(&self) -> String {
        format!("{}/work_tracker.db", self.data_dir.trim_end_matches('/'))
    }

    pub fn dl_db(&self) -> String {
        format!("{}/drivers_license.db", self.data_dir.trim_end_matches('/'))
    }

    pub fn output_dir(&self) -> String {
   		std::fs::create_dir_all(format!("{}/output", self.data_dir.trim_end_matches("/"))).ok();
        format!("{}/output", self.data_dir.trim_end_matches("/"))
    }

    pub fn accent(&self) -> egui::Color32 {
        let [r, g, b] = self.accent_color;
        egui::Color32::from_rgb(r, g, b)
    }

    pub fn row_height(&self) -> f32 {
        if self.compact_mode { 14.0 } else { 18.0 }
    }

    pub fn header_height(&self) -> f32 {
        if self.compact_mode { 16.0 } else { 20.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::date;

    #[test]
    fn date_format_iso() {
        assert_eq!(DateFormat::Iso.format(date(2024, 3, 5)), "2024-03-05");
    }

    #[test]
    fn date_format_european() {
        assert_eq!(DateFormat::European.format(date(2024, 3, 5)), "05.03.2024");
    }

    #[test]
    fn date_format_american() {
        assert_eq!(DateFormat::American.format(date(2024, 3, 5)), "03/05/2024");
    }

    #[test]
    fn work_db_path() {
        let s = AppSettings {
            data_dir: "/data".to_string(),
            ..Default::default()
        };
        assert_eq!(s.work_db(), "/data/work_tracker.db");
    }

    #[test]
    fn dl_db_path_strips_trailing_slash() {
        let s = AppSettings {
            data_dir: "/data/".to_string(),
            ..Default::default()
        };
        assert_eq!(s.dl_db(), "/data/drivers_license.db");
    }

    #[test]
    fn row_and_header_heights() {
        let normal = AppSettings {
            compact_mode: false,
            ..Default::default()
        };
        let compact = AppSettings {
            compact_mode: true,
            ..Default::default()
        };
        assert_eq!(normal.row_height(), 18.0);
        assert_eq!(compact.row_height(), 14.0);
        assert_eq!(normal.header_height(), 20.0);
        assert_eq!(compact.header_height(), 16.0);
    }
}
