use chrono::NaiveDate;
use egui::{Id, Modal};

use crate::work_tracker::{WorkEntry, WorkStats, WorkTracker};

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

fn metric_card(ui: &mut egui::Ui, label: &str, value: &str, sub: Option<&str>) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .inner_margin(egui::Margin::same(8))
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).small());
            ui.label(egui::RichText::new(value).heading());
            if let Some(s) = sub {
                ui.label(egui::RichText::new(s).small().weak());
            }
        });
}

fn bar_chart(ui: &mut egui::Ui, title: &str, rows: &[(String, usize)], max: usize) {
    ui.label(egui::RichText::new(title).small().weak());

    let label_w = rows
        .iter()
        .take(6)
        .map(|(name, _)| {
            ui.painter()
                .layout_no_wrap(
                    name.clone(),
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                )
                .size()
                .x
        })
        .fold(0.0_f32, f32::max)
        + 4.0;

    for (name, count) in rows.iter().take(6) {
        ui.horizontal(|ui| {
            ui.set_min_height(16.0);
            ui.add_sized(
                [label_w, 16.0],
                egui::Label::new(egui::RichText::new(name).small()),
            );
            let bar_w = ui.available_width() - 30.0;
            let filled = bar_w * (*count as f32 / max as f32);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, ui.visuals().faint_bg_color);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(filled, 8.0)),
                2.0,
                egui::Color32::from_rgb(0x1D, 0x9E, 0x75),
            );
            ui.label(egui::RichText::new(count.to_string()).small().weak());
        });
    }
}
