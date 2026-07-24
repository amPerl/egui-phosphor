// Every variant uses the same codepoints, so they share one table and each
// variant module re-exports it. `update.py` verifies this still holds.
mod codepoints;

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

pub mod bytes {
    #[cfg(feature = "thin")]
    pub mod thin {
        pub const FONT: &[u8] = include_bytes!("../../res/Phosphor-Thin.ttf");
    }
    #[cfg(feature = "light")]
    pub mod light {
        pub const FONT: &[u8] = include_bytes!("../../res/Phosphor-Light.ttf");
    }
    #[cfg(feature = "regular")]
    pub mod regular {
        pub const FONT: &[u8] = include_bytes!("../../res/Phosphor.ttf");
    }
    #[cfg(feature = "bold")]
    pub mod bold {
        pub const FONT: &[u8] = include_bytes!("../../res/Phosphor-Bold.ttf");
    }
    #[cfg(feature = "fill")]
    pub mod fill {
        pub const FONT: &[u8] = include_bytes!("../../res/Phosphor-Fill.ttf");
    }
}

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
    pub fn font_bytes(&self) -> &'static [u8] {
        match self {
            #[cfg(feature = "thin")]
            Variant::Thin => bytes::thin::FONT,
            #[cfg(feature = "light")]
            Variant::Light => bytes::light::FONT,
            #[cfg(feature = "regular")]
            Variant::Regular => bytes::regular::FONT,
            #[cfg(feature = "bold")]
            Variant::Bold => bytes::bold::FONT,
            #[cfg(feature = "fill")]
            Variant::Fill => bytes::fill::FONT,
        }
    }

    pub fn font_data(&self) -> egui::FontData {
        egui::FontData::from_static(self.font_bytes())
    }
}
