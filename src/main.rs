use crate::tabs::{HomeTab, SettingsTab, Tab, WorkTab};

mod tabs;
mod work_tracker;

#[derive(Debug)]
enum Page {
    Home(HomeTab),
    Work(WorkTab),
    Settings(SettingsTab),
}

#[derive(Debug)]
struct Tracker {
    page: Page,
}

impl Tracker {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);

        Self {
            page: Page::Home(HomeTab::default()),
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(matches!(self.page, Page::Home(_)), "Home")
                .clicked()
            {
                self.page = Page::Home(HomeTab::default());
            }

            if ui
                .selectable_label(matches!(self.page, Page::Work(_)), "Work")
                .clicked()
            {
                self.page = Page::Work(WorkTab::new());
            }

            if ui
                .selectable_label(matches!(self.page, Page::Settings(_)), "Settings")
                .clicked()
            {
                self.page = Page::Settings(SettingsTab::default());
            }
            if ui.button("Close").clicked() {
            	ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn show_page(&mut self, ui: &mut egui::Ui) {
        match &mut self.page {
            Page::Home(page) => page.ui(ui),
            Page::Work(page) => page.ui(ui),
            Page::Settings(page) => page.ui(ui),
        }
    }
}

impl eframe::App for Tracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            tracing::info!("Bye bye ...");
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.top_bar(ui);
            ui.separator();
            self.show_page(ui);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt().init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Tracker",
        options,
        Box::new(|cc| Ok(Box::new(Tracker::new(cc)))),
    );

    Ok(())
}
