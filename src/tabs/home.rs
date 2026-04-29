use chrono::{Datelike, NaiveDate};

use crate::{
    drivers_license_tracker::DriversLicenseTracker,
    settings::AppSettings,
    ui::metric_card,
    work_tracker::WorkTracker,
};

#[derive(Debug)]
enum RecentItem {
    WorkShift { date: NaiveDate, station: String, shift: String },
    Lesson { date: NaiveDate, lesson_type: String, instructor: String },
    Expense { date: NaiveDate, description: String, amount: f64 },
}

impl RecentItem {
    fn date(&self) -> NaiveDate {
        match self {
            Self::WorkShift { date, .. } | Self::Lesson { date, .. } | Self::Expense { date, .. } => *date,
        }
    }
    fn tag(&self) -> &str {
        match self {
            Self::WorkShift { .. } => "Work",
            Self::Lesson { .. } => "Lesson",
            Self::Expense { .. } => "Expense",
        }
    }
    fn tag_color(&self) -> egui::Color32 {
        match self {
            Self::WorkShift { .. } => egui::Color32::from_rgb(0x3B, 0x82, 0xF6),
            Self::Lesson { .. } => egui::Color32::from_rgb(0x1D, 0x9E, 0x75),
            Self::Expense { .. } => egui::Color32::from_rgb(0xF5, 0x9E, 0x0B),
        }
    }
}

#[derive(Debug)]
pub struct HomeTab {
    shifts_this_month: usize,
    last_shift: Option<NaiveDate>,
    total_lessons: usize,
    total_spent: f64,
    recent: Vec<RecentItem>,
}

impl HomeTab {
    pub fn new(work_db: &str, dl_db: &str) -> Self {
        let wt = WorkTracker::new(work_db);
        let dt = DriversLicenseTracker::new(dl_db);

        let work_entries = wt.load_all();
        let today = chrono::Local::now().date_naive();

        let shifts_this_month = work_entries
            .iter()
            .filter(|e| e.date.year() == today.year() && e.date.month() == today.month())
            .count();
        let last_shift = work_entries.first().map(|e| e.date);

        let dl_stats = dt.stats();

        let mut recent: Vec<RecentItem> = Vec::new();
        for e in work_entries.iter().take(5) {
            recent.push(RecentItem::WorkShift {
                date: e.date,
                station: e.station.clone(),
                shift: e.shift.clone(),
            });
        }
        for l in dt.load_all_lessons().iter().take(5) {
            recent.push(RecentItem::Lesson {
                date: l.date,
                lesson_type: l.lesson_type.clone(),
                instructor: l.instructor.clone(),
            });
        }
        for e in dt.load_all_expenses().iter().take(5) {
            recent.push(RecentItem::Expense {
                date: e.date,
                description: e.description.clone(),
                amount: e.amount,
            });
        }
        recent.sort_by_key(|b| std::cmp::Reverse(b.date()));
        recent.truncate(7);

        Self {
            shifts_this_month,
            last_shift,
            total_lessons: dl_stats.total_lessons,
            total_spent: dl_stats.total_spent,
            recent,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        ui.columns(2, |cols| {
            cols[0].label(egui::RichText::new("Work").weak());
            metric_card(
                &mut cols[0],
                "Shifts this month",
                &self.shifts_this_month.to_string(),
                None,
            );
            let last = self
                .last_shift
                .map(|d| settings.date_format.format(d))
                .unwrap_or_else(|| "—".to_string());
            metric_card(&mut cols[0], "Last shift", &last, None);

            cols[1].label(egui::RichText::new("Führerschein").weak());
            metric_card(
                &mut cols[1],
                "Total lessons",
                &self.total_lessons.to_string(),
                None,
            );
            metric_card(
                &mut cols[1],
                "Total spent",
                &format!("€{:.2}", self.total_spent),
                None,
            );
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Recent Activity").weak());
        ui.add_space(4.0);

        if self.recent.is_empty() {
            ui.label(egui::RichText::new("No entries yet.").small().weak());
            return;
        }

        for item in &self.recent {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(settings.date_format.format(item.date()))
                        .small()
                        .weak(),
                );
                egui::Frame::new()
                    .fill(item.tag_color())
                    .inner_margin(egui::Margin::same(3))
                    .corner_radius(3.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(item.tag())
                                .small()
                                .color(egui::Color32::WHITE),
                        );
                    });
                let detail = match item {
                    RecentItem::WorkShift { station, shift, .. } => {
                        format!("{station} · {shift}")
                    }
                    RecentItem::Lesson { lesson_type, instructor, .. } => {
                        format!("{lesson_type} · {instructor}")
                    }
                    RecentItem::Expense { description, amount, .. } => {
                        format!("{description} · €{amount:.2}")
                    }
                };
                ui.label(egui::RichText::new(detail).small());
            });
        }
    }
}
