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
