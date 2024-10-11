use egui::RichText;

fn main() {
    eframe::run_native(
        "egui-phosphor simple-demo",
        Default::default(),
        Box::new(|cc| Ok(Box::new(Demo::new(cc)))),
    )
    .unwrap();
}

struct Demo {}

impl Demo {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        Self {}
    }
}

impl eframe::App for Demo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(
                RichText::new(format!("Hello, world! {}", egui_phosphor::regular::ACORN))
                    .size(32.0),
            );
        });
    }
}
