pub mod variants;
pub use variants::*;

#[cfg(feature = "subset")]
pub mod subset;

/// Re-exported so [`subset!`] can name `egui` types without assuming anything
/// about the calling crate's imports.
pub use egui;

pub fn add_to_fonts(fonts: &mut egui::FontDefinitions, variant: Variant) {
    add_font_bytes_to_fonts(fonts, "phosphor", variant.font_bytes());
}

/// Insert `bytes` under `name`, make it a fallback for proportional text so
/// icons can be mixed into ordinary labels, and register a
/// [`FontFamily::Name(name)`](egui::FontFamily::Name) that resolves to it.
///
/// `name` must be unique per font, otherwise later calls overwrite earlier ones.
///
/// Because every Phosphor variant uses the same codepoints, proportional text
/// can only resolve an icon to one of them. Use [`add_font_bytes_as_family`] for
/// any additional variants and select them explicitly by family.
pub fn add_font_bytes_to_fonts(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    bytes: &'static [u8],
) {
    add_font_bytes_as_family(fonts, name, bytes);

    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        font_keys.insert(1, name.into());
    }
}

/// Insert `bytes` under `name` and register a
/// [`FontFamily::Name(name)`](egui::FontFamily::Name) for it, without adding it
/// to the proportional fonts.
///
/// The family also contains the proportional fonts. `name` goes second, not
/// first: the subset maps `a`-`z` to the blank glyphs Phosphor's ligatures are
/// built from, so putting it ahead of the text font would blank out ordinary
/// lowercase text. Second still puts it ahead of any other icon font and of
/// egui's emoji fonts, so the icons themselves resolve from it.
pub fn add_font_bytes_as_family(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    bytes: &'static [u8],
) {
    fonts
        .font_data
        .insert(name.into(), egui::FontData::from_static(bytes).into());

    let mut keys = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    keys.insert(keys.len().min(1), name.into());
    fonts
        .families
        .insert(egui::FontFamily::Name(name.into()), keys);
}
