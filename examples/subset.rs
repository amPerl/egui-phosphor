//! Compile-time subsetting: only the icons named below are embedded.
//!
//! Run with `cargo run --example subset --features subset`.

use egui::{RichText, ViewportBuilder};

egui_phosphor::subset! {
    /// The only Phosphor icons this example embeds.
    pub mod icons {
        use regular::{GEAR, HOUSE, USER, TRASH, HEART, BELL};
        use fill::{GEAR, HOUSE, USER, TRASH, HEART, BELL};
        use thin::{GEAR, HOUSE, USER, TRASH, HEART, BELL};
    }
}

const SHOWN: &[(&str, &str)] = &[
    ("GEAR", icons::regular::GEAR),
    ("HOUSE", icons::regular::HOUSE),
    ("USER", icons::regular::USER),
    ("TRASH", icons::regular::TRASH),
    ("HEART", icons::regular::HEART),
    ("BELL", icons::regular::BELL),
];

fn main() {
    eframe::run_native(
        "egui-phosphor subset demo",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size((460.0, 480.0)),
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
        // Regular becomes the fallback for proportional text, so its icons can
        // be dropped straight into ordinary labels. The other two are reachable
        // through their own families.
        icons::regular::add_to_fonts(&mut fonts);
        icons::fill::add_as_family(&mut fonts);
        icons::thin::add_as_family(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);
        Self {}
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        use egui_phosphor::Variant;

        let embedded =
            icons::regular::FONT.len() + icons::fill::FONT.len() + icons::thin::FONT.len();
        let full = Variant::Regular.font_bytes().len()
            + Variant::Fill.font_bytes().len()
            + Variant::Thin.font_bytes().len();

        ui.vertical(|ui| {
            ui.heading("Subsetted Phosphor");
            ui.label(format!(
                "{} icons from 3 variants: {embedded} bytes embedded instead of {full} \
                 ({:.0}x smaller)",
                SHOWN.len() * 3,
                full as f32 / embedded as f32
            ));
            ui.separator();

            // Every variant uses the same codepoints, so the constants below are
            // all the same character -- the font family is what picks the weight.
            egui::Grid::new("icons")
                .spacing((16.0, 8.0))
                .show(ui, |ui| {
                    ui.label("");
                    for v in ["regular", "fill", "thin"] {
                        ui.label(RichText::new(v).weak());
                    }
                    ui.end_row();

                    for (name, icon) in SHOWN {
                        ui.label(*name);
                        ui.label(RichText::new(*icon).size(28.0));
                        ui.label(icons::fill::rich(*icon).size(28.0));
                        ui.label(icons::thin::rich(*icon).size(28.0));
                        ui.end_row();
                    }
                });

            ui.separator();
            ui.label("Mixed with text:");
            ui.label(format!("{} Settings", icons::regular::GEAR));
            ui.label(icons::fill::rich(format!(
                "{} Favourite",
                icons::fill::HEART
            )));
            ui.label(icons::thin::rich(format!(
                "{} Notifications",
                icons::thin::BELL
            )));
        });
    }
}
