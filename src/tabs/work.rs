use chrono::NaiveDate;
use egui::{Id, Modal};
use egui_material_icons::icons::{
    ICON_ADD, ICON_DELETE, ICON_DOWNLOAD, ICON_EDIT, ICON_FILE_OPEN,
};

use crate::{
    settings::AppSettings,
    ui::{ExportStatus, bar_chart, icon_label, metric_card},
    work_tracker::{WorkEntry, WorkStats, WorkTracker},
};

#[derive(Debug)]
pub struct WorkTab {
    work_tracker: WorkTracker,

    // add modal
    add_entry_modal: bool,
    new_date_entry: String,
    new_station_entry: String,
    new_shift_entry: String,

    // edit modal
    editing_id: Option<i64>,
    edit_date: String,
    edit_station: String,
    edit_shift: String,

    // convert modal
    convert_modal: bool,
    pdf_path: String,
    convert_csv: bool,
    convert_ics: bool,

    // shared validation error for whichever modal is open
    modal_error: Option<String>,

    cache: Vec<WorkEntry>,
    dirty: bool,
    confirm_delete: Option<i64>,

    // filter
    filter_text: String,
    filter_from: String,
    filter_to: String,

    station_suggestions: Vec<String>,
    shift_suggestions: Vec<String>,

    stats: WorkStats,
    export_status: ExportStatus,
}

impl WorkTab {
    pub fn new(db_path: &str) -> Self {
        Self {
            work_tracker: WorkTracker::new(db_path),
            add_entry_modal: false,
            new_date_entry: String::new(),
            new_station_entry: String::new(),
            new_shift_entry: String::new(),
            editing_id: None,
            edit_date: String::new(),
            edit_station: String::new(),
            edit_shift: String::new(),
            convert_modal: false,
            pdf_path: String::new(),
            convert_csv: false,
            convert_ics: false,
            modal_error: None,
            cache: Vec::new(),
            dirty: true,
            confirm_delete: None,
            filter_text: String::new(),
            filter_from: String::new(),
            filter_to: String::new(),
            station_suggestions: Vec::new(),
            shift_suggestions: Vec::new(),
            stats: WorkStats::default(),
            export_status: ExportStatus::default(),
        }
    }

    fn work_entry_table(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        self.reload_cache();

        // Action bar
        ui.horizontal(|ui| {
            if ui.button(icon_label(ICON_ADD, "New Entry")).clicked() {
                self.add_entry_modal = true;
            }
            if ui.button(icon_label(ICON_DOWNLOAD, "Export CSV")).clicked() {
                let path = format!("{}/work_export.csv", settings.data_dir);
                match self.work_tracker.export_csv(&path) {
                    Ok(()) => self.export_status.set(format!("Exported to {path}")),
                    Err(e) => self.export_status.set(format!("Error: {e}")),
                }
            }
            if let Some(status) = self.export_status.tick() {
                ui.label(egui::RichText::new(status).small().weak());
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs(1));
            }
            if ui
                .button(icon_label(ICON_FILE_OPEN, "Convert Dienstplan (CSV/ICS)"))
                .clicked()
            {
                self.convert_modal = true;
            }
        });

        // Convert Modal
        if self.convert_modal {
            let modal = Modal::new(Id::new("convert_modal")).show(ui.ctx(), |ui| {
                ui.heading("Convert Dienstplan PDF to ICS / CSV");

                if let Some(err) = &self.modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.horizontal(|ui| {
                    // Show shortened path so the label doesn't blow out the modal width
                    let display = if self.pdf_path.is_empty() {
                        "No file selected".to_string()
                    } else {
                        std::path::Path::new(&self.pdf_path)
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| self.pdf_path.clone())
                    };
                    ui.label(display);
                    ui.add_space(8.0);
                    if ui.button(icon_label(ICON_FILE_OPEN, "Browse…")).clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .pick_file()
                    {
                        self.pdf_path = path.to_string_lossy().to_string();
                        self.modal_error = None;
                    }
                });

                ui.add_space(4.0);
                // Labels were swapped in the original — fixed here:
                ui.checkbox(&mut self.convert_csv, "Export CSV");
                ui.checkbox(&mut self.convert_ics, "Export ICS (calendar)");

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let ready = !self.pdf_path.is_empty() && (self.convert_csv || self.convert_ics);
                    ui.add_enabled_ui(ready, |ui| {
                        if ui.button(icon_label(ICON_DOWNLOAD, "Convert")).clicked() {
                            let result =
                                crate::dienstplan::convert(crate::dienstplan::ConvertOptions {
                                    pdf_path: std::path::PathBuf::from(&self.pdf_path),
                                    output_dir: None, // writes next to the PDF
                                    write_csv: self.convert_csv,
                                    write_ics: self.convert_ics,
                                    event_prefix: "Dienst".to_string(),
                                });
                            match result {
                                Err(e) => {
                                    self.modal_error = Some(format!("Conversion failed: {e}"));
                                }
                                Ok(res) => {
                                    let mut parts = vec![format!(
                                        "Done — {} shifts for {}",
                                        res.shift_count, res.mitarbeiter
                                    )];
                                    if let Some(p) = &res.csv_path {
                                        parts.push(format!("CSV: {}", p.display()));
                                    }
                                    if let Some(p) = &res.ics_path {
                                        parts.push(format!("ICS: {}", p.display()));
                                    }
                                    self.export_status.set(parts.join("\n"));
                                    self.modal_error = None;
                                    self.convert_modal = false;
                                    self.pdf_path = String::new();
                                    self.convert_csv = false;
                                    self.convert_ics = false;
                                }
                            }
                        }
                    });
                });
            });

            if modal.should_close() {
                self.modal_error = None;
                self.convert_modal = false;
                self.pdf_path = String::new();
                self.convert_csv = false;
                self.convert_ics = false;
            }
        }

        // Add modal
        if self.add_entry_modal {
            let station_suggestions = self.station_suggestions.clone();
            let shift_suggestions = self.shift_suggestions.clone();
            let modal = Modal::new(Id::new("work_add_modal")).show(ui.ctx(), |ui| {
                ui.heading("New Work Entry");
                if let Some(err) = &self.modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

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
                if !station_suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &station_suggestions {
                            if ui.small_button(s).clicked() {
                                self.new_station_entry = s.clone();
                            }
                        }
                    });
                }
                ui.add(egui::TextEdit::singleline(&mut self.new_shift_entry).hint_text("Shift"));
                if !shift_suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &shift_suggestions {
                            if ui.small_button(s).clicked() {
                                self.new_shift_entry = s.clone();
                            }
                        }
                    });
                }
                ui.separator();
                if ui.button("Add").clicked() {
                    match NaiveDate::parse_from_str(&self.new_date_entry, "%Y-%m-%d") {
                        Err(_) => {
                            self.modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        Ok(date) => {
                            self.work_tracker.add(
                                date,
                                self.new_station_entry.clone().as_str(),
                                self.new_shift_entry.clone().as_str(),
                            );
                            self.dirty = true;
                            self.modal_error = None;
                            self.new_date_entry.clear();
                            self.new_station_entry.clear();
                            self.new_shift_entry.clear();
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.add_entry_modal = false;
                self.modal_error = None;
            }
        }

        // Edit modal
        if self.editing_id.is_some() {
            let station_suggestions = self.station_suggestions.clone();
            let shift_suggestions = self.shift_suggestions.clone();
            let modal = Modal::new(Id::new("work_edit_modal")).show(ui.ctx(), |ui| {
                ui.heading("Edit Work Entry");
                if let Some(err) = &self.modal_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_date)
                            .hint_text("Date (YYYY-MM-DD)"),
                    );
                    if ui.button("Today").clicked() {
                        self.edit_date = chrono::Local::now().date_naive().to_string();
                    }
                });

                ui.add(egui::TextEdit::singleline(&mut self.edit_station).hint_text("Station"));
                if !station_suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &station_suggestions {
                            if ui.small_button(s).clicked() {
                                self.edit_station = s.clone();
                            }
                        }
                    });
                }
                ui.add(egui::TextEdit::singleline(&mut self.edit_shift).hint_text("Shift"));
                if !shift_suggestions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for s in &shift_suggestions {
                            if ui.small_button(s).clicked() {
                                self.edit_shift = s.clone();
                            }
                        }
                    });
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    match NaiveDate::parse_from_str(&self.edit_date, "%Y-%m-%d") {
                        Err(_) => {
                            self.modal_error = Some("Invalid date. Use YYYY-MM-DD.".into());
                        }
                        Ok(date) => {
                            let id = self.editing_id.unwrap();
                            self.work_tracker.update(
                                id,
                                date,
                                self.edit_station.clone().as_str(),
                                self.edit_shift.clone().as_str(),
                            );
                            self.dirty = true;
                            self.modal_error = None;
                            self.editing_id = None;
                            ui.close();
                        }
                    }
                }
            });

            if modal.should_close() {
                self.editing_id = None;
                self.modal_error = None;
            }
        }

        // Filter bar
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.filter_text)
                    .hint_text("search…")
                    .desired_width(120.0),
            );
            ui.label("From:");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter_from)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            ui.label("To:");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter_to)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(90.0),
            );
            if ui.small_button("Clear").clicked() {
                self.filter_text.clear();
                self.filter_from.clear();
                self.filter_to.clear();
            }
        });

        ui.separator();

        let row_h = settings.row_height();
        let hdr_h = settings.header_height();

        let q = self.filter_text.to_lowercase();
        let from_date = NaiveDate::parse_from_str(&self.filter_from, "%Y-%m-%d").ok();
        let to_date = NaiveDate::parse_from_str(&self.filter_to, "%Y-%m-%d").ok();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::remainder())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(hdr_h, |mut header| {
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
                    for entry in self.cache.iter().filter(|e| {
                        let text_match = q.is_empty()
                            || e.station.to_lowercase().contains(&q)
                            || e.shift.to_lowercase().contains(&q);
                        let from_match = from_date.is_none_or(|d| e.date >= d);
                        let to_match = to_date.is_none_or(|d| e.date <= d);
                        text_match && from_match && to_match
                    }) {
                        body.row(row_h, |mut row| {
                            row.col(|ui| {
                                ui.label(settings.date_format.format(entry.date));
                            });
                            row.col(|ui| {
                                ui.label(&entry.station);
                            });
                            row.col(|ui| {
                                ui.label(&entry.shift);
                            });
                            row.col(|ui| {
                                if ui.button(ICON_EDIT).on_hover_text("Edit").clicked() {
                                    self.editing_id = Some(entry.id);
                                    self.edit_date = entry.date.format("%Y-%m-%d").to_string();
                                    self.edit_station = entry.station.clone();
                                    self.edit_shift = entry.shift.clone();
                                    self.modal_error = None;
                                }
                                if ui.button(ICON_DELETE).on_hover_text("Delete").clicked() {
                                    self.confirm_delete = Some(entry.id);
                                }
                            });
                        });
                    }
                });
        });

        // Delete confirmation modal
        if let Some(id) = self.confirm_delete {
            let mut do_delete = false;
            let modal = Modal::new(Id::new("work_confirm_delete")).show(ui.ctx(), |ui| {
                ui.heading("Delete entry?");
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
                self.work_tracker.delete(id);
                self.dirty = true;
                self.confirm_delete = None;
                self.reload_cache();
            } else if modal.should_close() {
                self.confirm_delete = None;
            }
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
            bar_chart(
                &mut cols[0],
                "By station",
                &s.by_station,
                max_station,
                accent,
            );
            bar_chart(&mut cols[1], "By shift", &s.by_shift, max_shift, accent);
        });
    }

    fn reload_cache(&mut self) {
        if self.dirty {
            self.cache = self.work_tracker.load_all();
            self.stats = self.work_tracker.stats();
            self.station_suggestions = self.work_tracker.unique_stations();
            self.shift_suggestions = self.work_tracker.unique_shifts();
            self.dirty = false;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        if !self.work_tracker.is_connected() {
            ui.colored_label(
                egui::Color32::from_rgb(0xF5, 0x9E, 0x0B),
                "No database — set a valid data directory in Settings and restart.",
            );
            return;
        }
        self.work_entry_stats(ui, settings);
        ui.separator();
        self.work_entry_table(ui, settings);
    }
}
