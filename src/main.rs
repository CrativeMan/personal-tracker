use egui_material_icons::icons::{
    ICON_CLOSE, ICON_DIRECTIONS_CAR, ICON_HOME, ICON_SETTINGS, ICON_WORK,
};

use crate::{
    settings::AppSettings,
    tabs::{DriverslicenseTab, HomeTab, SettingsTab, WorkTab},
    ui::icon_label,
};

mod drivers_license_tracker;
mod settings;
mod tabs;
mod ui;
mod work_tracker;

#[derive(Debug)]
enum Page {
    Home(Box<HomeTab>),
    Work(Box<WorkTab>),
    Führerschein(Box<DriverslicenseTab>),
    Settings(SettingsTab),
}

#[derive(Debug)]
struct Tracker {
    page: Page,
    settings: AppSettings,
}

impl Tracker {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);
        let settings = AppSettings::load();
        settings.apply(&cc.egui_ctx);

        let home = HomeTab::new(&settings.work_db(), &settings.dl_db());
        Self {
            page: Page::Home(Box::new(home)),
            settings,
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let work_db = self.settings.work_db();
        let dl_db = self.settings.dl_db();
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    matches!(self.page, Page::Home(_)),
                    icon_label(ICON_HOME, "Home"),
                )
                .clicked()
            {
                self.page = Page::Home(Box::new(HomeTab::new(&work_db, &dl_db)));
            }

            if ui
                .selectable_label(
                    matches!(self.page, Page::Work(_)),
                    icon_label(ICON_WORK, "Work"),
                )
                .clicked()
            {
                self.page = Page::Work(Box::new(WorkTab::new(&work_db)));
            }

            if ui
                .selectable_label(
                    matches!(self.page, Page::Führerschein(_)),
                    icon_label(ICON_DIRECTIONS_CAR, "Führerschein"),
                )
                .clicked()
            {
                self.page = Page::Führerschein(Box::new(DriverslicenseTab::new(&dl_db)));
            }

            if ui
                .selectable_label(
                    matches!(self.page, Page::Settings(_)),
                    icon_label(ICON_SETTINGS, "Settings"),
                )
                .clicked()
            {
                self.page = Page::Settings(SettingsTab::default());
            }
            if ui.button(icon_label(ICON_CLOSE, "Close")).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn show_page(&mut self, ui: &mut egui::Ui) {
        let Tracker { page, settings } = self;
        match page {
            Page::Home(page) => page.ui(ui, settings),
            Page::Work(page) => page.ui(ui, settings),
            Page::Führerschein(page) => page.ui(ui, settings),
            Page::Settings(page) => page.ui(ui, settings),
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Tracker",
        options,
        Box::new(|cc| Ok(Box::new(Tracker::new(cc)))),
    );

    Ok(())
}
