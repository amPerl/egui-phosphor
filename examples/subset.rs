//! Compile-time subsetting: only the icons named below are embedded.
//!
//! Run with `cargo run --example subset --features subset`.

use egui::ViewportBuilder;

egui_phosphor::subset! {
    /// The only Phosphor icons this example embeds.
    pub mod icons {
        use regular::{GEAR, HOUSE, USER, TRASH, HEART, BELL, CHECK, FOLDER, WARNING, CALENDAR};
    }
}

fn main() {
    eframe::run_native(
        "egui-phosphor subset demo",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size((420.0, 420.0)),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Demo::new(cc)))),
    )
    .unwrap();
}

struct Demo {}

impl Demo {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        icons::regular::add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);
        Self {}
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.vertical(|ui| {
            let full = egui_phosphor::Variant::Regular.font_bytes().len();
            ui.heading("Subsetted Phosphor");
            ui.label(format!(
                "{} bytes embedded instead of {full} ({:.1}x smaller)",
                icons::regular::FONT.len(),
                full as f32 / icons::regular::FONT.len() as f32
            ));
            ui.separator();

            for (name, icon) in [
                ("GEAR", icons::regular::GEAR),
                ("HOUSE", icons::regular::HOUSE),
                ("USER", icons::regular::USER),
                ("TRASH", icons::regular::TRASH),
                ("HEART", icons::regular::HEART),
                ("BELL", icons::regular::BELL),
                ("CHECK", icons::regular::CHECK),
                ("FOLDER", icons::regular::FOLDER),
                ("WARNING", icons::regular::WARNING),
                ("CALENDAR", icons::regular::CALENDAR),
            ] {
                ui.horizontal(|ui| {
                    for size in [14.0, 20.0, 32.0] {
                        ui.label(egui::RichText::new(icon).size(size));
                    }
                    ui.label(name);
                });
            }
        });
    }
}
