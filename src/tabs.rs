use chrono::NaiveDate;
use egui::{Id, Modal};

use crate::{
    drivers_license_tracker::{DriversLicenseStats, DriversLicenseTracker, ExpenseEntry, LessonEntry},
    ui::{bar_chart, bar_chart_money, metric_card},
    work_tracker::{WorkEntry, WorkStats, WorkTracker},
};

pub trait Tab {
    fn ui(&mut self, ui: &mut egui::Ui);
}

#[derive(Debug, Default)]
pub struct HomeTab {}

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
}

impl WorkTab {
    pub fn new() -> Self {
        Self {
            work_tracker: WorkTracker::new("./work_tracker.db"),
            add_entry_modal: false,
            new_date_entry: String::new(),
            new_station_entry: String::new(),
            new_shift_entry: String::new(),
            cache: Vec::new(),
            dirty: true,
            to_delete: None,
            stats: WorkStats::default(),
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
}

impl DriverslicenseTab {
    pub fn new() -> Self {
        Self {
            tracker: DriversLicenseTracker::new("./drivers_license.db"),
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

    fn stats_ui(&self, ui: &mut egui::Ui) {
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

        ui.columns(2, |cols| {
            bar_chart(&mut cols[0], "By lesson type", &s.by_lesson_type, max_type);
            bar_chart_money(&mut cols[1], "By expense category", &s.by_category, max_cat);
        });
    }

    fn lessons_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New Lesson").clicked() {
                self.add_lesson_modal = true;
            }
        });

        if self.add_lesson_modal {
            let suggestions = self.lesson_type_suggestions.clone();
            let modal = Modal::new(Id::new("dl_lesson_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Lesson");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_lesson_date).hint_text("Date"),
                    );
                    if ui.button("Today").clicked() {
                        self.new_lesson_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.new_lesson_type)
                        .hint_text("Lesson type"),
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
                    if let Ok(date) =
                        NaiveDate::parse_from_str(&self.new_lesson_date, "%Y-%m-%d")
                    {
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Type"); });
                    header.col(|ui| { ui.label("Instructor"); });
                    header.col(|ui| { ui.label("Notes"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    for entry in &self.lesson_cache {
                        body.row(18.0, |mut row| {
                            row.col(|ui| { ui.label(entry.date.to_string()); });
                            row.col(|ui| { ui.label(&entry.lesson_type); });
                            row.col(|ui| { ui.label(&entry.instructor); });
                            row.col(|ui| { ui.label(&entry.notes); });
                            row.col(|ui| {
                                if ui.button("Delete").clicked() {
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

    fn expenses_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New Expense").clicked() {
                self.add_expense_modal = true;
            }
        });

        if self.add_expense_modal {
            let suggestions = self.expense_category_suggestions.clone();
            let modal = Modal::new(Id::new("dl_expense_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Expense");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_expense_date)
                            .hint_text("Date"),
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Description"); });
                    header.col(|ui| { ui.label("Amount"); });
                    header.col(|ui| { ui.label("Category"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    for entry in &self.expense_cache {
                        body.row(18.0, |mut row| {
                            row.col(|ui| { ui.label(entry.date.to_string()); });
                            row.col(|ui| { ui.label(&entry.description); });
                            row.col(|ui| { ui.label(format!("€{:.2}", entry.amount)); });
                            row.col(|ui| { ui.label(&entry.category); });
                            row.col(|ui| {
                                if ui.button("Delete").clicked() {
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

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.reload_cache();
        self.stats = self.tracker.stats();
        self.stats_ui(ui);
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
            DlView::Lessons => self.lessons_ui(ui),
            DlView::Expenses => self.expenses_ui(ui),
        }
    }
}

#[derive(Debug, Default)]
pub struct SettingsTab {}

impl Tab for HomeTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label("To quit CTRL + Q");
        });
    }
}

impl WorkTab {
    fn work_entry_table(&mut self, ui: &mut egui::Ui) {
        self.reload_cache();
        // ---- ADD NEW ENTRY ----
        ui.horizontal(|ui| {
            if ui.button("New Entry").clicked() {
                self.add_entry_modal = true;
            }
        });

        if self.add_entry_modal {
            let modal = Modal::new(Id::new("Modal A")).show(ui.ctx(), |ui| {
                ui.heading("New Work Entry");

                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.new_date_entry).hint_text("Date"));

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

        // ---- TABLE ----
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Date");
                    });
                    header.col(|ui| {
                        ui.label("Station");
                    });
                    header.col(|ui| {
                        ui.label("Shift");
                    });
                    header.col(|ui| {
                        ui.label("Actions");
                    });
                })
                .body(|mut body| {
                    for entry in &self.cache {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(entry.date.to_string());
                            });

                            row.col(|ui| {
                                ui.label(&entry.station);
                            });

                            row.col(|ui| {
                                ui.label(&entry.shift);
                            });

                            row.col(|ui| {
                                if ui.button("Delete").clicked() {
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

    fn work_entry_stats(&mut self, ui: &mut egui::Ui) {
        let s = &self.stats;

        // ── top metric cards ──────────────────────────────────────────
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

        // ── bar charts ────────────────────────────────────────────────
        let max_station = s.by_station.first().map(|(_, n)| *n).unwrap_or(1);
        let max_shift = s.by_shift.first().map(|(_, n)| *n).unwrap_or(1);

        ui.columns(2, |cols| {
            bar_chart(&mut cols[0], "By station", &s.by_station, max_station);
            bar_chart(&mut cols[1], "By shift", &s.by_shift, max_shift);
        });
    }

    fn reload_cache(&mut self) {
        if self.dirty {
            self.cache = self.work_tracker.load_all();
            self.stats = self.work_tracker.stats();
            self.dirty = false;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.work_entry_stats(ui);
        ui.separator();
        self.work_entry_table(ui);
    }
}

impl Tab for SettingsTab {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
}
