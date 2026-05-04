use crate::settings::{AppSettings, DateFormat, ACCENT_PALETTE};

#[derive(Debug, Default)]
pub struct SettingsTab {}

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
            if ui.button("Browse…").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_directory(&settings.data_dir)
                    .pick_folder()
                {
                    settings.data_dir = path.to_string_lossy().to_string();
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
