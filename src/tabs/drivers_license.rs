use chrono::NaiveDate;
use egui::{Id, Modal};
use egui_material_icons::icons::{ICON_ADD, ICON_DELETE, ICON_DOWNLOAD, ICON_EDIT};

use crate::{
    drivers_license_tracker::{DriversLicenseStats, DriversLicenseTracker, ExpenseEntry, LessonEntry},
    settings::AppSettings,
    ui::{bar_chart, bar_chart_money, icon_label, metric_card, ExportStatus},
};

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
    lesson_confirm_delete: Option<i64>,

    // lesson add modal
    add_lesson_modal: bool,
    new_lesson_date: String,
    new_lesson_type: String,
    new_lesson_instructor: String,
    new_lesson_notes: String,
    lesson_type_suggestions: Vec<String>,

    // lesson edit modal
    editing_lesson_id: Option<i64>,
    edit_lesson_date: String,
    edit_lesson_type: String,
    edit_lesson_instructor: String,
    edit_lesson_notes: String,

    lesson_modal_error: Option<String>,

    // lesson filter
    lesson_filter_text: String,
    lesson_filter_from: String,
    lesson_filter_to: String,

    expense_cache: Vec<ExpenseEntry>,
    expense_dirty: bool,
    expense_confirm_delete: Option<i64>,

    // expense add modal
    add_expense_modal: bool,
    new_expense_date: String,
    new_expense_description: String,
    new_expense_amount: String,
    new_expense_category: String,
    expense_category_suggestions: Vec<String>,

    // expense edit modal
    editing_expense_id: Option<i64>,
    edit_expense_date: String,
    edit_expense_description: String,
    edit_expense_amount: String,
    edit_expense_category: String,

    expense_modal_error: Option<String>,

    // expense filter
    expense_filter_text: String,
    expense_filter_from: String,
    expense_filter_to: String,

    stats: DriversLicenseStats,
    export_status: ExportStatus,
}

impl DriverslicenseTab {
    pub fn new(db_path: &str) -> Self {
        Self {
            tracker: DriversLicenseTracker::new(db_path),
            view: DlView::Lessons,
            lesson_cache: Vec::new(),
            lesson_dirty: true,
            lesson_confirm_delete: None,
            add_lesson_modal: false,
            new_lesson_date: String::new(),
            new_lesson_type: String::new(),
            new_lesson_instructor: String::new(),
            new_lesson_notes: String::new(),
            lesson_type_suggestions: Vec::new(),
            editing_lesson_id: None,
            edit_lesson_date: String::new(),
            edit_lesson_type: String::new(),
            edit_lesson_instructor: String::new(),
            edit_lesson_notes: String::new(),
            lesson_modal_error: None,
            lesson_filter_text: String::new(),
            lesson_filter_from: String::new(),
            lesson_filter_to: String::new(),
            expense_cache: Vec::new(),
            expense_dirty: true,
            expense_confirm_delete: None,
            add_expense_modal: false,
            new_expense_date: String::new(),
            new_expense_description: String::new(),
            new_expense_amount: String::new(),
            new_expense_category: String::new(),
            expense_category_suggestions: Vec::new(),
            editing_expense_id: None,
            edit_expense_date: String::new(),
            edit_expense_description: String::new(),
            edit_expense_amount: String::new(),
            edit_expense_category: String::new(),
            expense_modal_error: None,
            expense_filter_text: String::new(),
            expense_filter_from: String::new(),
            expense_filter_to: String::new(),
            stats: DriversLicenseStats::default(),
            export_status: ExportStatus::default(),
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
        // Action bar
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Lesson")).clicked() {
                self.add_lesson_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/lessons_export.csv", settings.data_dir);
                match self.tracker.export_lessons_csv(&path) {
                    Ok(()) => self.export_status.set(format!("Exported to {path}")),
                    Err(e) => self.export_status.set(format!("Error: {e}")),
                }
            }
            if let Some(status) = self.export_status.tick() {
                ui.label(egui::RichText::new(status).small().weak());
                ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
            }
        });

        // Add modal
        if self.add_lesson_modal {
            let suggestions = self.lesson_type_suggestions.clone();
            let modal = Modal::new(Id::new("dl_lesson_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Lesson");
                if let Some(err) = &self.lesson_modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

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
                    match NaiveDate::parse_from_str(&self.new_lesson_date, "%Y-%m-%d") {
                        Err(_) => {
                            self.lesson_modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        Ok(date) => {
                            self.tracker.add_lesson(
                                date,
                                &self.new_lesson_type.clone(),
                                &self.new_lesson_instructor.clone(),
                                &self.new_lesson_notes.clone(),
                            );
                            self.lesson_dirty = true;
                            self.expense_dirty = true;
                            self.lesson_modal_error = None;
                            self.new_lesson_date.clear();
                            self.new_lesson_type.clear();
                            self.new_lesson_instructor.clear();
                            self.new_lesson_notes.clear();
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.add_lesson_modal = false;
                self.lesson_modal_error = None;
            }
        }

        // Edit modal
        if self.editing_lesson_id.is_some() {
            let suggestions = self.lesson_type_suggestions.clone();
            let modal = Modal::new(Id::new("dl_lesson_edit_modal")).show(ui.ctx(), |ui| {
                ui.heading("Edit Lesson");
                if let Some(err) = &self.lesson_modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_lesson_date)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.edit_lesson_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_lesson_type).hint_text("Lesson type"),
                );
                if !suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &suggestions {
                            if ui.small_button(s).clicked() {
                                self.edit_lesson_type = s.clone();
                            }
                        }
                    });
                }

                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_lesson_instructor)
                        .hint_text("Instructor"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_lesson_notes).hint_text("Notes"),
                );

                ui.separator();
                if ui.button("Save").clicked() {
                    match NaiveDate::parse_from_str(&self.edit_lesson_date, "%Y-%m-%d") {
                        Err(_) => {
                            self.lesson_modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        Ok(date) => {
                            let id = self.editing_lesson_id.unwrap();
                            self.tracker.update_lesson(
                                id,
                                date,
                                &self.edit_lesson_type.clone(),
                                &self.edit_lesson_instructor.clone(),
                                &self.edit_lesson_notes.clone(),
                            );
                            self.lesson_dirty = true;
                            self.lesson_modal_error = None;
                            self.editing_lesson_id = None;
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.editing_lesson_id = None;
                self.lesson_modal_error = None;
            }
        }

        // Filter bar
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.lesson_filter_text)
                    .hint_text("search…")
                    .desired_width(120.0),
            );
            ui.label("From:");
            ui.add(
                egui::TextEdit::singleline(&mut self.lesson_filter_from)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            ui.label("To:");
            ui.add(
                egui::TextEdit::singleline(&mut self.lesson_filter_to)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            if ui.small_button("Clear").clicked() {
                self.lesson_filter_text.clear();
                self.lesson_filter_from.clear();
                self.lesson_filter_to.clear();
            }
        });

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        let q = self.lesson_filter_text.to_lowercase();
        let from_date = NaiveDate::parse_from_str(&self.lesson_filter_from, "%Y-%m-%d").ok();
        let to_date = NaiveDate::parse_from_str(&self.lesson_filter_to, "%Y-%m-%d").ok();

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
                    for entry in self.lesson_cache.iter().filter(|e| {
                        let text_match = q.is_empty()
                            || e.lesson_type.to_lowercase().contains(&q)
                            || e.instructor.to_lowercase().contains(&q)
                            || e.notes.to_lowercase().contains(&q);
                        let from_match = from_date.is_none_or(|d| e.date >= d);
                        let to_match = to_date.is_none_or(|d| e.date <= d);
                        text_match && from_match && to_match
                    }) {
                        body.row(row_h, |mut row| {
                            row.col(|ui| { ui.label(settings.date_format.format(entry.date)); });
                            row.col(|ui| { ui.label(&entry.lesson_type); });
                            row.col(|ui| { ui.label(&entry.instructor); });
                            row.col(|ui| { ui.label(&entry.notes); });
                            row.col(|ui| {
                                if ui.button(ICON_EDIT).on_hover_text("Edit").clicked() {
                                    self.editing_lesson_id = Some(entry.id);
                                    self.edit_lesson_date = entry.date.format("%Y-%m-%d").to_string();
                                    self.edit_lesson_type = entry.lesson_type.clone();
                                    self.edit_lesson_instructor = entry.instructor.clone();
                                    self.edit_lesson_notes = entry.notes.clone();
                                    self.lesson_modal_error = None;
                                }
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.lesson_confirm_delete = Some(entry.id);
                                }
                            });
                        });
                    }
                });
        });

        // Delete confirmation modal
        if let Some(id) = self.lesson_confirm_delete {
            let mut do_delete = false;
            let modal = Modal::new(Id::new("dl_lesson_confirm_delete")).show(ui.ctx(), |ui| {
                ui.heading("Delete lesson?");
                ui.label("This cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        do_delete = true;
                        ui.close();
                    }
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                });
            });
            if do_delete {
                self.tracker.delete_lesson(id);
                self.lesson_dirty = true;
                self.expense_dirty = true;
                self.lesson_confirm_delete = None;
                self.lesson_cache = self.tracker.load_all_lessons();
                self.lesson_type_suggestions = self.tracker.unique_lesson_types();
                self.stats = self.tracker.stats();
            } else if modal.should_close() {
                self.lesson_confirm_delete = None;
            }
        }
    }

    fn expenses_ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        // Action bar
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Expense")).clicked() {
                self.add_expense_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/expenses_export.csv", settings.data_dir);
                match self.tracker.export_expenses_csv(&path) {
                    Ok(()) => self.export_status.set(format!("Exported to {path}")),
                    Err(e) => self.export_status.set(format!("Error: {e}")),
                }
            }
            if let Some(status) = self.export_status.tick() {
                ui.label(egui::RichText::new(status).small().weak());
                ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
            }
        });

        // Add modal
        if self.add_expense_modal {
            let suggestions = self.expense_category_suggestions.clone();
            let modal = Modal::new(Id::new("dl_expense_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Expense");
                if let Some(err) = &self.expense_modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

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
                    let date_result = NaiveDate::parse_from_str(&self.new_expense_date, "%Y-%m-%d");
                    let amount_result = self.new_expense_amount.parse::<f64>();
                    match (date_result, amount_result) {
                        (Err(_), _) => {
                            self.expense_modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        (_, Err(_)) => {
                            self.expense_modal_error = Some("Amount must be a number.".into());
                        }
                        (Ok(date), Ok(amount)) => {
                            self.tracker.add_expense(
                                date,
                                &self.new_expense_description.clone(),
                                amount,
                                &self.new_expense_category.clone(),
                            );
                            self.expense_dirty = true;
                            self.expense_modal_error = None;
                            self.new_expense_date.clear();
                            self.new_expense_description.clear();
                            self.new_expense_amount.clear();
                            self.new_expense_category.clear();
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.add_expense_modal = false;
                self.expense_modal_error = None;
            }
        }

        // Edit modal
        if self.editing_expense_id.is_some() {
            let suggestions = self.expense_category_suggestions.clone();
            let modal = Modal::new(Id::new("dl_expense_edit_modal")).show(ui.ctx(), |ui| {
                ui.heading("Edit Expense");
                if let Some(err) = &self.expense_modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_expense_date)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.edit_expense_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_expense_description)
                        .hint_text("Description"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_expense_amount)
                        .hint_text("Amount (€)"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_expense_category)
                        .hint_text("Category"),
                );
                if !suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &suggestions {
                            if ui.small_button(s).clicked() {
                                self.edit_expense_category = s.clone();
                            }
                        }
                    });
                }

                ui.separator();
                if ui.button("Save").clicked() {
                    let date_result = NaiveDate::parse_from_str(&self.edit_expense_date, "%Y-%m-%d");
                    let amount_result = self.edit_expense_amount.parse::<f64>();
                    match (date_result, amount_result) {
                        (Err(_), _) => {
                            self.expense_modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        (_, Err(_)) => {
                            self.expense_modal_error = Some("Amount must be a number.".into());
                        }
                        (Ok(date), Ok(amount)) => {
                            let id = self.editing_expense_id.unwrap();
                            self.tracker.update_expense(
                                id,
                                date,
                                &self.edit_expense_description.clone(),
                                amount,
                                &self.edit_expense_category.clone(),
                            );
                            self.expense_dirty = true;
                            self.expense_modal_error = None;
                            self.editing_expense_id = None;
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.editing_expense_id = None;
                self.expense_modal_error = None;
            }
        }

        // Filter bar
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.expense_filter_text)
                    .hint_text("search…")
                    .desired_width(120.0),
            );
            ui.label("From:");
            ui.add(
                egui::TextEdit::singleline(&mut self.expense_filter_from)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            ui.label("To:");
            ui.add(
                egui::TextEdit::singleline(&mut self.expense_filter_to)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            if ui.small_button("Clear").clicked() {
                self.expense_filter_text.clear();
                self.expense_filter_from.clear();
                self.expense_filter_to.clear();
            }
        });

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        let q = self.expense_filter_text.to_lowercase();
        let from_date = NaiveDate::parse_from_str(&self.expense_filter_from, "%Y-%m-%d").ok();
        let to_date = NaiveDate::parse_from_str(&self.expense_filter_to, "%Y-%m-%d").ok();

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
                    for entry in self.expense_cache.iter().filter(|e| {
                        let text_match = q.is_empty()
                            || e.description.to_lowercase().contains(&q)
                            || e.category.to_lowercase().contains(&q);
                        let from_match = from_date.is_none_or(|d| e.date >= d);
                        let to_match = to_date.is_none_or(|d| e.date <= d);
                        text_match && from_match && to_match
                    }) {
                        body.row(row_h, |mut row| {
                            row.col(|ui| { ui.label(settings.date_format.format(entry.date)); });
                            row.col(|ui| { ui.label(&entry.description); });
                            row.col(|ui| { ui.label(format!("€{:.2}", entry.amount)); });
                            row.col(|ui| { ui.label(&entry.category); });
                            row.col(|ui| {
                                if ui.button(ICON_EDIT).on_hover_text("Edit").clicked() {
                                    self.editing_expense_id = Some(entry.id);
                                    self.edit_expense_date = entry.date.format("%Y-%m-%d").to_string();
                                    self.edit_expense_description = entry.description.clone();
                                    self.edit_expense_amount = format!("{:.2}", entry.amount);
                                    self.edit_expense_category = entry.category.clone();
                                    self.expense_modal_error = None;
                                }
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.expense_confirm_delete = Some(entry.id);
                                }
                            });
                        });
                    }
                });
        });

        // Delete confirmation modal
        if let Some(id) = self.expense_confirm_delete {
            let mut do_delete = false;
            let modal = Modal::new(Id::new("dl_expense_confirm_delete")).show(ui.ctx(), |ui| {
                ui.heading("Delete expense?");
                ui.label("This cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        do_delete = true;
                        ui.close();
                    }
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                });
            });
            if do_delete {
                self.tracker.delete_expense(id);
                self.expense_dirty = true;
                self.expense_confirm_delete = None;
                self.expense_cache = self.tracker.load_all_expenses();
                self.expense_category_suggestions = self.tracker.unique_expense_categories();
                self.stats = self.tracker.stats();
            } else if modal.should_close() {
                self.expense_confirm_delete = None;
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        if !self.tracker.is_connected() {
            ui.colored_label(
                egui::Color32::from_rgb(0xF5, 0x9E, 0x0B),
                "No database — set a valid data directory in Settings and restart.",
            );
            return;
        }
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
