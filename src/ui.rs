pub fn metric_card(ui: &mut egui::Ui, label: &str, value: &str, sub: Option<&str>) {
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

pub fn bar_chart(
    ui: &mut egui::Ui,
    title: &str,
    rows: &[(String, usize)],
    max: usize,
    accent: egui::Color32,
) {
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
                accent,
            );
            ui.label(egui::RichText::new(count.to_string()).small().weak());
        });
    }
}

pub fn bar_chart_money(
    ui: &mut egui::Ui,
    title: &str,
    rows: &[(String, f64)],
    max: f64,
    accent: egui::Color32,
) {
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

    for (name, amount) in rows.iter().take(6) {
        ui.horizontal(|ui| {
            ui.set_min_height(16.0);
            ui.add_sized(
                [label_w, 16.0],
                egui::Label::new(egui::RichText::new(name).small()),
            );
            let bar_w = ui.available_width() - 52.0;
            let filled = if max > 0.0 {
                bar_w * (*amount as f32 / max as f32)
            } else {
                0.0
            };
            let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, ui.visuals().faint_bg_color);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(filled, 8.0)),
                2.0,
                accent,
            );
            ui.label(egui::RichText::new(format!("€{amount:.0}")).small().weak());
        });
    }
}
