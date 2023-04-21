pub mod icons;
pub use icons::*;

pub fn font_data() -> egui::FontData {
    let mut font_data = egui::FontData::from_static(include_bytes!("../res/Phosphor.ttf"));
    font_data.tweak.y_offset_factor = 0.0;
    font_data
}

pub fn add_to_fonts(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert("phosphor".into(), font_data());

    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        font_keys.push("phosphor".into());
    }
}
