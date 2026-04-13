use chrono::NaiveDate;
use egui::{Id, Modal};

use crate::work_tracker::{WorkEntry, WorkTracker};

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

    cache: Vec<WorkEntry>,
    to_delete: Option<i64>,
}

impl WorkTab {
    pub fn new() -> Self {
        Self {
            work_tracker: WorkTracker::new("./work_tracker.db"),
            add_entry_modal: false,
            new_date_entry: String::new(),
            new_station_entry: String::new(),
            cache: Vec::new(),
            to_delete: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct SettingsTab {}

impl Tab for HomeTab {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
}

impl WorkTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
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
                ui.separator();
                if ui.button("Add").clicked() {
                    if let Ok(date) = NaiveDate::parse_from_str(&self.new_date_entry, "%Y-%m-%d") {
                        self.work_tracker
                            .add(date, self.new_station_entry.clone().as_str());
                    }
                    ui.close();
                    self.new_date_entry.clear();
                    self.new_station_entry.clear();
                    self.reload_cache();
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
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Date");
                    });
                    header.col(|ui| {
                        ui.label("Station");
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
                                if ui.button("Delete").clicked() {
                                    self.to_delete = Some(entry.id);
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
    fn reload_cache(&mut self) {
        self.cache = self.work_tracker.load_all();
    }
}

impl Tab for SettingsTab {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
}
