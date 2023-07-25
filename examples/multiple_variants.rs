use egui_phosphor::{bold, fill, light, regular, thin};

fn main() {
    eframe::run_native(
        "egui-phosphor demo",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 755.0)),
            ..Default::default()
        },
        Box::new(|cc| Box::new(Demo::new(cc))),
    )
    .unwrap();
}

struct Demo {}

impl Demo {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "phosphor-thin".into(),
            egui_phosphor::Variant::Thin.font_data(),
        );
        fonts.families.insert(
            egui::FontFamily::Name("phosphor-thin".into()),
            vec!["Ubuntu-Light".into(), "phosphor-thin".into()],
        );

        fonts.font_data.insert(
            "phosphor-light".into(),
            egui_phosphor::Variant::Light.font_data(),
        );
        fonts.families.insert(
            egui::FontFamily::Name("phosphor-light".into()),
            vec!["Ubuntu-Light".into(), "phosphor-light".into()],
        );

        fonts.font_data.insert(
            "phosphor".into(),
            egui_phosphor::Variant::Regular.font_data(),
        );
        fonts.families.insert(
            egui::FontFamily::Name("phosphor".into()),
            vec!["Ubuntu-Light".into(), "phosphor".into()],
        );

        fonts.font_data.insert(
            "phosphor-bold".into(),
            egui_phosphor::Variant::Bold.font_data(),
        );
        fonts.families.insert(
            egui::FontFamily::Name("phosphor-bold".into()),
            vec!["Ubuntu-Light".into(), "phosphor-bold".into()],
        );

        fonts.font_data.insert(
            "phosphor-fill".into(),
            egui_phosphor::Variant::Fill.font_data(),
        );
        fonts.families.insert(
            egui::FontFamily::Name("phosphor-fill".into()),
            vec!["Ubuntu-Light".into(), "phosphor-fill".into()],
        );

        cc.egui_ctx.set_fonts(fonts);

        Self {}
    }
}

impl eframe::App for Demo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for (family, icon) in [
                ("phosphor-thin", thin::FILE_CODE),
                ("phosphor-light", light::FILE_CODE),
                ("phosphor", regular::FILE_CODE),
                ("phosphor-bold", bold::FILE_CODE),
                ("phosphor-fill", fill::FILE_CODE),
            ] {
                ui.heading(family);
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    for size in [16.0, 32.0, 48.0] {
                        let demo_text = format!("FILE_CODE {icon}");
                        ui.label(
                            egui::RichText::new(&demo_text)
                                .family(egui::FontFamily::Name(family.into()))
                                .size(size),
                        );
                    }
                });
            }
        });
    }
}
