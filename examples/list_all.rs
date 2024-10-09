fn main() {
    eframe::run_native(
        "egui-phosphor demo",
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
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut icons = egui_phosphor::regular::ICONS.to_vec();
                    icons.sort_by_key(|(_, icon)| {
                        let code = icon.chars().next().unwrap() as usize;
                        format!("{code:#04X}")
                    });
                    for (icon_name, icon) in icons {
                        let code = icon.chars().next().unwrap() as usize;
                        ui.label(
                            egui::RichText::new(format!("{code:#04X} {icon_name} {icon}",))
                                .size(16.0),
                        );
                    }
                });
        });
    }
}
