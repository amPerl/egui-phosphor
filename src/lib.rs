pub mod icons;
pub use icons::*;

pub fn font_data() -> egui::FontData {
    #[cfg(all(feature = "foo", feature = "bar"))]
    compile_error!("Features \"lines\" and \"filled\" are mutually exclusive ");

    let mut font_data;
    #[cfg(feature = "lines")]
    {
       font_data = egui::FontData::from_static(include_bytes!("../res/Phosphor.ttf"));
    }
    #[cfg(feature = "filled")]
    {
       font_data = egui::FontData::from_static(include_bytes!("../res/Phosphor-Fill.ttf"));
    }
    font_data.tweak.y_offset_factor = 0.0;
    font_data
}

pub fn add_to_fonts(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert("phosphor".into(), font_data());

    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        font_keys.push("phosphor".into());
    }
}
