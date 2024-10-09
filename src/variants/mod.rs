#[cfg(feature = "bold")]
pub mod bold;
#[cfg(feature = "fill")]
pub mod fill;
#[cfg(feature = "light")]
pub mod light;
#[cfg(feature = "regular")]
pub mod regular;
#[cfg(feature = "thin")]
pub mod thin;

#[cfg(not(any(
    feature = "thin",
    feature = "light",
    feature = "regular",
    feature = "bold",
    feature = "fill",
)))]
compile_error!(
    "At least one font variant must be selected as a crate feature. When in doubt, use default features."
);

#[derive(Debug, Clone, Copy)]
pub enum Variant {
    #[cfg(feature = "thin")]
    Thin,
    #[cfg(feature = "light")]
    Light,
    #[cfg(feature = "regular")]
    Regular,
    #[cfg(feature = "bold")]
    Bold,
    #[cfg(feature = "fill")]
    Fill,
}

impl Variant {
    #[must_use]
    pub const fn font_bytes(&self) -> &'static [u8] {
        match self {
            #[cfg(feature = "thin")]
            Self::Thin => &*include_bytes!("../../res/Phosphor-Thin.ttf"),
            #[cfg(feature = "light")]
            Self::Light => &*include_bytes!("../../res/Phosphor-Light.ttf"),
            #[cfg(feature = "regular")]
            Self::Regular => &*include_bytes!("../../res/Phosphor.ttf"),
            #[cfg(feature = "bold")]
            Self::Bold => &*include_bytes!("../../res/Phosphor-Bold.ttf"),
            #[cfg(feature = "fill")]
            Self::Fill => &*include_bytes!("../../res/Phosphor-Fill.ttf"),
        }
    }

    #[must_use]
    pub fn font_data(&self) -> egui::FontData {
        egui::FontData::from_static(self.font_bytes())
    }
}
