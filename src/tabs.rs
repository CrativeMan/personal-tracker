use chrono::{Datelike, NaiveDate};
use egui::{Id, Modal};
use egui_material_icons::icons::{ICON_ADD, ICON_DELETE, ICON_DOWNLOAD};

use crate::{
    drivers_license_tracker::{DriversLicenseStats, DriversLicenseTracker, ExpenseEntry, LessonEntry},
    settings::{AppSettings, ACCENT_PALETTE},
    ui::{bar_chart, bar_chart_money, icon_label, metric_card},
    work_tracker::{WorkEntry, WorkStats, WorkTracker},
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
        recent.sort_by(|a, b| b.date().cmp(&a.date()));
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

#[derive(Debug)]
pub struct WorkTab {
    work_tracker: WorkTracker,
    add_entry_modal: bool,
    new_date_entry: String,
    new_station_entry: String,
    new_shift_entry: String,

    cache: Vec<WorkEntry>,
    dirty: bool,
    to_delete: Option<i64>,

    stats: WorkStats,
    export_status: Option<String>,
}

impl WorkTab {
    pub fn new(db_path: &str) -> Self {
        Self {
            work_tracker: WorkTracker::new(db_path),
            add_entry_modal: false,
            new_date_entry: String::new(),
            new_station_entry: String::new(),
            new_shift_entry: String::new(),
            cache: Vec::new(),
            dirty: true,
            to_delete: None,
            stats: WorkStats::default(),
            export_status: None,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
enum DlView {
    #[default]
    Lessons,
    Expenses,
}

#[derive(Debug)]
pub struct DriverslicenseTab {
    tracker: DriversLicenseTracker,
    view: DlView,

    lesson_cache: Vec<LessonEntry>,
    lesson_dirty: bool,
    lesson_to_delete: Option<i64>,
    add_lesson_modal: bool,
    new_lesson_date: String,
    new_lesson_type: String,
    new_lesson_instructor: String,
    new_lesson_notes: String,
    lesson_type_suggestions: Vec<String>,

    expense_cache: Vec<ExpenseEntry>,
    expense_dirty: bool,
    expense_to_delete: Option<i64>,
    add_expense_modal: bool,
    new_expense_date: String,
    new_expense_description: String,
    new_expense_amount: String,
    new_expense_category: String,
    expense_category_suggestions: Vec<String>,

    stats: DriversLicenseStats,
    export_status: Option<String>,
}

impl DriverslicenseTab {
    pub fn new(db_path: &str) -> Self {
        Self {
            tracker: DriversLicenseTracker::new(db_path),
            view: DlView::Lessons,
            lesson_cache: Vec::new(),
            lesson_dirty: true,
            lesson_to_delete: None,
            add_lesson_modal: false,
            new_lesson_date: String::new(),
            new_lesson_type: String::new(),
            new_lesson_instructor: String::new(),
            new_lesson_notes: String::new(),
            lesson_type_suggestions: Vec::new(),
            expense_cache: Vec::new(),
            expense_dirty: true,
            expense_to_delete: None,
            add_expense_modal: false,
            new_expense_date: String::new(),
            new_expense_description: String::new(),
            new_expense_amount: String::new(),
            new_expense_category: String::new(),
            expense_category_suggestions: Vec::new(),
            stats: DriversLicenseStats::default(),
            export_status: None,
        }
    }

    fn reload_cache(&mut self) {
        if self.lesson_dirty {
            self.lesson_cache = self.tracker.load_all_lessons();
            self.lesson_type_suggestions = self.tracker.unique_lesson_types();
            self.lesson_dirty = false;
        }
        if self.expense_dirty {
            self.expense_cache = self.tracker.load_all_expenses();
            self.expense_category_suggestions = self.tracker.unique_expense_categories();
            self.expense_dirty = false;
        }
        if self.lesson_dirty || self.expense_dirty {
            self.stats = self.tracker.stats();
        }
    }

    fn stats_ui(&self, ui: &mut egui::Ui, settings: &AppSettings) {
        let s = &self.stats;
        ui.horizontal(|ui| {
            metric_card(ui, "Total lessons", &s.total_lessons.to_string(), None);
            metric_card(ui, "This month", &s.lessons_this_month.to_string(), None);
            metric_card(ui, "Total spent", &format!("€{:.2}", s.total_spent), None);
            metric_card(
                ui,
                "Spent this month",
                &format!("€{:.2}", s.spent_this_month),
                None,
            );
        });

        ui.add_space(8.0);

        let max_type = s.by_lesson_type.first().map(|(_, n)| *n).unwrap_or(1);
        let max_cat = s.by_category.first().map(|(_, n)| *n).unwrap_or(1.0);
        let accent = settings.accent();

        ui.columns(2, |cols| {
            bar_chart(&mut cols[0], "By lesson type", &s.by_lesson_type, max_type, accent);
            bar_chart_money(&mut cols[1], "By expense category", &s.by_category, max_cat, accent);
        });
    }

    fn lessons_ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Lesson")).clicked() {
                self.add_lesson_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/lessons_export.csv", settings.data_dir);
                match self.tracker.export_lessons_csv(&path) {
                    Ok(()) => self.export_status = Some(format!("Exported to {path}")),
                    Err(e) => self.export_status = Some(format!("Error: {e}")),
                }
            }
            if let Some(status) = &self.export_status {
                ui.label(egui::RichText::new(status).small().weak());
            }
        });

        if self.add_lesson_modal {
            let suggestions = self.lesson_type_suggestions.clone();
            let modal = Modal::new(Id::new("dl_lesson_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Lesson");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_lesson_date)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.new_lesson_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.new_lesson_type).hint_text("Lesson type"),
                );
                if !suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &suggestions {
                            if ui.small_button(s).clicked() {
                                self.new_lesson_type = s.clone();
                            }
                        }
                    });
                }

                ui.add(
                    egui::TextEdit::singleline(&mut self.new_lesson_instructor)
                        .hint_text("Instructor"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_lesson_notes).hint_text("Notes"),
                );

                ui.separator();
                if ui.button("Add").clicked() {
                    if let Ok(date) = NaiveDate::parse_from_str(&self.new_lesson_date, "%Y-%m-%d") {
                        self.tracker.add_lesson(
                            date,
                            &self.new_lesson_type.clone(),
                            &self.new_lesson_instructor.clone(),
                            &self.new_lesson_notes.clone(),
                        );
                        self.lesson_dirty = true;
                        self.expense_dirty = true;
                    }
                    ui.close();
                    self.new_lesson_date.clear();
                    self.new_lesson_type.clear();
                    self.new_lesson_instructor.clear();
                    self.new_lesson_notes.clear();
                }
            });

            if modal.should_close() {
                self.add_lesson_modal = false;
            }
        }

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .header(hdr_h, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Type"); });
                    header.col(|ui| { ui.label("Instructor"); });
                    header.col(|ui| { ui.label("Notes"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    for entry in &self.lesson_cache {
                        body.row(row_h, |mut row| {
                            row.col(|ui| { ui.label(settings.date_format.format(entry.date)); });
                            row.col(|ui| { ui.label(&entry.lesson_type); });
                            row.col(|ui| { ui.label(&entry.instructor); });
                            row.col(|ui| { ui.label(&entry.notes); });
                            row.col(|ui| {
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.lesson_to_delete = Some(entry.id);
                                    self.lesson_dirty = true;
                                    self.expense_dirty = true;
                                }
                            });
                        });
                    }
                });
        });

        if let Some(id) = self.lesson_to_delete.take() {
            self.tracker.delete_lesson(id);
            self.lesson_cache = self.tracker.load_all_lessons();
            self.lesson_type_suggestions = self.tracker.unique_lesson_types();
            self.stats = self.tracker.stats();
        }
    }

    fn expenses_ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Expense")).clicked() {
                self.add_expense_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/expenses_export.csv", settings.data_dir);
                match self.tracker.export_expenses_csv(&path) {
                    Ok(()) => self.export_status = Some(format!("Exported to {path}")),
                    Err(e) => self.export_status = Some(format!("Error: {e}")),
                }
            }
            if let Some(status) = &self.export_status {
                ui.label(egui::RichText::new(status).small().weak());
            }
        });

        if self.add_expense_modal {
            let suggestions = self.expense_category_suggestions.clone();
            let modal = Modal::new(Id::new("dl_expense_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Expense");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_expense_date)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.new_expense_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.new_expense_description)
                        .hint_text("Description"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_expense_amount)
                        .hint_text("Amount (€)"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_expense_category)
                        .hint_text("Category"),
                );
                if !suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &suggestions {
                            if ui.small_button(s).clicked() {
                                self.new_expense_category = s.clone();
                            }
                        }
                    });
                }

                ui.separator();
                if ui.button("Add").clicked() {
                    if let (Ok(date), Ok(amount)) = (
                        NaiveDate::parse_from_str(&self.new_expense_date, "%Y-%m-%d"),
                        self.new_expense_amount.parse::<f64>(),
                    ) {
                        self.tracker.add_expense(
                            date,
                            &self.new_expense_description.clone(),
                            amount,
                            &self.new_expense_category.clone(),
                        );
                        self.expense_dirty = true;
                    }
                    ui.close();
                    self.new_expense_date.clear();
                    self.new_expense_description.clear();
                    self.new_expense_amount.clear();
                    self.new_expense_category.clear();
                }
            });

            if modal.should_close() {
                self.add_expense_modal = false;
            }
        }

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(hdr_h, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Description"); });
                    header.col(|ui| { ui.label("Amount"); });
                    header.col(|ui| { ui.label("Category"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    for entry in &self.expense_cache {
                        body.row(row_h, |mut row| {
                            row.col(|ui| { ui.label(settings.date_format.format(entry.date)); });
                            row.col(|ui| { ui.label(&entry.description); });
                            row.col(|ui| { ui.label(format!("€{:.2}", entry.amount)); });
                            row.col(|ui| { ui.label(&entry.category); });
                            row.col(|ui| {
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.expense_to_delete = Some(entry.id);
                                    self.expense_dirty = true;
                                }
                            });
                        });
                    }
                });
        });

        if let Some(id) = self.expense_to_delete.take() {
            self.tracker.delete_expense(id);
            self.expense_cache = self.tracker.load_all_expenses();
            self.expense_category_suggestions = self.tracker.unique_expense_categories();
            self.stats = self.tracker.stats();
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        self.reload_cache();
        self.stats = self.tracker.stats();
        self.stats_ui(ui, settings);
        ui.separator();

        ui.horizontal(|ui| {
            if ui.selectable_label(self.view == DlView::Lessons, "Lessons").clicked() {
                self.view = DlView::Lessons;
            }
            if ui.selectable_label(self.view == DlView::Expenses, "Expenses").clicked() {
                self.view = DlView::Expenses;
            }
        });
        ui.separator();

        match self.view {
            DlView::Lessons => self.lessons_ui(ui, settings),
            DlView::Expenses => self.expenses_ui(ui, settings),
        }
    }
}

#[derive(Debug, Default)]
pub struct SettingsTab {}

impl WorkTab {
    fn work_entry_table(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        self.reload_cache();
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Entry")).clicked() {
                self.add_entry_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/work_export.csv", settings.data_dir);
                match self.work_tracker.export_csv(&path) {
                    Ok(()) => self.export_status = Some(format!("Exported to {path}")),
                    Err(e) => self.export_status = Some(format!("Error: {e}")),
                }
            }
            if let Some(status) = &self.export_status {
                ui.label(egui::RichText::new(status).small().weak());
            }
        });

        if self.add_entry_modal {
            let modal = Modal::new(Id::new("Modal A")).show(ui.ctx(), |ui| {
                ui.heading("New Work Entry");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_date_entry)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.new_date_entry = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.new_station_entry).hint_text("Station"),
                );
                ui.add(egui::TextEdit::singleline(&mut self.new_shift_entry).hint_text("Shift"));
                ui.separator();
                if ui.button("Add").clicked() {
                    if let Ok(date) = NaiveDate::parse_from_str(&self.new_date_entry, "%Y-%m-%d") {
                        self.work_tracker.add(
                            date,
                            self.new_station_entry.clone().as_str(),
                            self.new_shift_entry.clone().as_str(),
                        );
                    }
                    ui.close();
                    self.new_date_entry.clear();
                    self.new_station_entry.clear();
                    self.new_shift_entry.clear();
                    self.dirty = true;
                }
            });

            if modal.should_close() {
                self.add_entry_modal = false;
            }
        }

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(hdr_h, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Station"); });
                    header.col(|ui| { ui.label("Shift"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    for entry in &self.cache {
                        body.row(row_h, |mut row| {
                            row.col(|ui| {
                                ui.label(settings.date_format.format(entry.date));
                            });
                            row.col(|ui| { ui.label(&entry.station); });
                            row.col(|ui| { ui.label(&entry.shift); });
                            row.col(|ui| {
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.to_delete = Some(entry.id);
                                    self.dirty = true;
                                }
                            });
                        });
                    }
                });
        });

        if let Some(id) = self.to_delete {
            self.work_tracker.delete(id);
            self.reload_cache();
        }
    }

    fn work_entry_stats(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        let s = &self.stats;

        ui.horizontal(|ui| {
            metric_card(ui, "Total shifts", &s.total_shifts.to_string(), None);
            metric_card(ui, "This month", &s.shifts_this_month.to_string(), None);
            metric_card(ui, "Unique stations", &s.unique_stations.to_string(), None);
            if let Some((name, count)) = &s.most_common_shift {
                metric_card(
                    ui,
                    "Most common shift",
                    name,
                    Some(format!("{count} times").as_str()),
                );
            }
        });

        ui.add_space(8.0);

        let max_station = s.by_station.first().map(|(_, n)| *n).unwrap_or(1);
        let max_shift = s.by_shift.first().map(|(_, n)| *n).unwrap_or(1);
        let accent = settings.accent();

        ui.columns(2, |cols| {
            bar_chart(&mut cols[0], "By station", &s.by_station, max_station, accent);
            bar_chart(&mut cols[1], "By shift", &s.by_shift, max_shift, accent);
        });
    }

    fn reload_cache(&mut self) {
        if self.dirty {
            self.cache = self.work_tracker.load_all();
            self.stats = self.work_tracker.stats();
            self.dirty = false;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        self.work_entry_stats(ui, settings);
        ui.separator();
        self.work_entry_table(ui, settings);
    }
}

impl SettingsTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        let mut changed = false;

        ui.heading("Settings");
        ui.add_space(8.0);

        // ── Theme ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Theme");
            ui.add_space(8.0);
            if ui.selectable_label(settings.dark_mode, "Dark").clicked() {
                settings.dark_mode = true;
                changed = true;
            }
            if ui.selectable_label(!settings.dark_mode, "Light").clicked() {
                settings.dark_mode = false;
                changed = true;
            }
        });

        ui.add_space(4.0);

        // ── Font size ──────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Font size");
            ui.add_space(8.0);
            if ui
                .add(
                    egui::Slider::new(&mut settings.pixels_per_point, 0.75..=2.0).step_by(0.25),
                )
                .changed()
            {
                changed = true;
            }
        });

        ui.add_space(4.0);

        // ── Date format ────────────────────────────────────────────────
        ui.horizontal(|ui| {
            use crate::settings::DateFormat;
            ui.label("Date format");
            ui.add_space(8.0);
            for fmt in [DateFormat::Iso, DateFormat::European, DateFormat::American] {
                if ui
                    .selectable_label(settings.date_format == fmt, fmt.label())
                    .clicked()
                {
                    settings.date_format = fmt;
                    changed = true;
                }
            }
        });

        ui.add_space(4.0);

        // ── Accent color ───────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Accent color");
            ui.add_space(8.0);
            for swatch in ACCENT_PALETTE {
                let color = egui::Color32::from_rgb(swatch[0], swatch[1], swatch[2]);
                let selected = settings.accent_color == swatch;
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 4.0, color);
                if selected {
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                }
                if resp.clicked() {
                    settings.accent_color = swatch;
                    changed = true;
                }
            }
        });

        ui.add_space(4.0);

        // ── Compact mode ───────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Compact mode");
            ui.add_space(8.0);
            if ui.checkbox(&mut settings.compact_mode, "").changed() {
                changed = true;
            }
        });

        ui.add_space(4.0);

        // ── Data directory ─────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Data directory");
            ui.add_space(8.0);
            if ui
                .add(egui::TextEdit::singleline(&mut settings.data_dir).desired_width(160.0))
                .lost_focus()
            {
                changed = true;
            }
        });
        ui.label(
            egui::RichText::new("Restart the app to apply a new data directory.")
                .small()
                .weak(),
        );

        if changed {
            settings.apply(ui.ctx());
            settings.save();
        }
    }
}
