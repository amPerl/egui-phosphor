//! Two variants of the *same* icon in one app.
//!
//! Every Phosphor variant uses the same codepoints, so `regular::GEAR` and
//! `fill::GEAR` are the same `&str`. Only the font family can distinguish them.

use egui::{FontDefinitions, FontFamily, FontId};

egui_phosphor::subset! {
    pub mod icons {
        use regular::{GEAR, HOUSE};
        use fill::{GEAR, HEART};
    }
}

/// Lays out `s` at 32px in `family`, which is what `RichText::family(..)`
/// resolves to internally.
fn raster(ctx: &egui::Context, s: &str, family: FontFamily) -> Option<u64> {
    let galley = ctx.fonts_mut(|f| {
        f.layout_no_wrap(
            s.to_owned(),
            FontId::new(32.0, family),
            egui::Color32::WHITE,
        )
    });
    let glyph = galley.rows.first()?.glyphs.first()?;
    let uv = glyph.uv_rect;
    let (x0, y0) = (uv.min[0] as usize, uv.min[1] as usize);
    let (x1, y1) = (uv.max[0] as usize, uv.max[1] as usize);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let img = ctx.fonts(|f| f.image());
    let mut ink = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            ink += img[(x, y)].a() as u64;
        }
    }
    Some(ink)
}

/// Reference rendering of `s` from a full, unsubsetted variant.
fn reference(bytes: &'static [u8], s: &str) -> u64 {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("r".to_owned(), egui::FontData::from_static(bytes).into());
    fonts
        .families
        .insert(FontFamily::Name("r".into()), vec!["r".to_owned()]);
    let ctx = egui::Context::default();
    ctx.set_fonts(fonts);
    let _ = ctx.run_ui(Default::default(), |_| {});
    raster(&ctx, s, FontFamily::Name("r".into())).expect("reference glyph must render")
}

fn app_ctx() -> egui::Context {
    let mut fonts = FontDefinitions::default();
    icons::regular::add_to_fonts(&mut fonts); // inline with ordinary text
    icons::fill::add_as_family(&mut fonts); // selected explicitly
    let ctx = egui::Context::default();
    ctx.set_fonts(fonts);
    let _ = ctx.run_ui(Default::default(), |_| {});
    ctx
}

#[test]
fn overlapping_icon_renders_the_requested_weight() {
    let ctx = app_ctx();

    // The two constants really are the same character.
    assert_eq!(icons::regular::GEAR, icons::fill::GEAR);

    let regular = reference(
        egui_phosphor::Variant::Regular.font_bytes(),
        icons::regular::GEAR,
    );
    let fill = reference(egui_phosphor::Variant::Fill.font_bytes(), icons::fill::GEAR);
    assert_ne!(regular, fill, "the two weights should differ");

    // Unqualified text resolves to whichever variant called `add_to_fonts`.
    assert_eq!(
        raster(&ctx, icons::regular::GEAR, FontFamily::Proportional),
        Some(regular),
        "unqualified GEAR should render the regular weight"
    );

    // Pinned to the filled family.
    assert_eq!(
        raster(&ctx, icons::fill::GEAR, icons::fill::family()),
        Some(fill),
        "GEAR pinned to icons::fill::family() should render the fill weight"
    );
}

#[test]
fn non_overlapping_icons_work_unqualified() {
    let ctx = app_ctx();
    let house = reference(
        egui_phosphor::Variant::Regular.font_bytes(),
        icons::regular::HOUSE,
    );
    assert_eq!(
        raster(&ctx, icons::regular::HOUSE, FontFamily::Proportional),
        Some(house)
    );
}

#[test]
fn rich_pins_the_family() {
    assert_eq!(
        icons::fill::rich("x"),
        egui::RichText::new("x").family(icons::fill::family())
    );
    // and it accepts the `format!` output the crate's usage is built around
    let mixed = icons::fill::rich(format!("{} Settings", icons::fill::GEAR));
    assert_eq!(mixed.text(), format!("{} Settings", icons::fill::GEAR));
}

#[test]
fn ordinary_text_still_renders_in_an_icon_family() {
    let ctx = app_ctx();
    // The family includes the proportional fonts, so text is not tofu.
    assert!(raster(&ctx, "A", icons::fill::family()).is_some());
    // lowercase: Phosphor maps a-z to blank ligature-input glyphs
    assert_eq!(
        raster(&ctx, "s", icons::fill::family()),
        raster(&ctx, "s", FontFamily::Proportional),
        "lowercase text in an icon family should match normal text"
    );
}
