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
    new_shift_entry: String,

    cache: Vec<WorkEntry>,
    dirty: bool,
    to_delete: Option<i64>,
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
                            &self.new_shift_entry.clone().as_str(),
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
    fn reload_cache(&mut self) {
        if self.dirty {
            self.cache = self.work_tracker.load_all();
            self.dirty = false;
        }
    }
}

impl Tab for SettingsTab {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
}
