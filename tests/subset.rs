//! Verifies that a subsetted font renders *identically* to the full variant.

use egui::{FontData, FontDefinitions, FontFamily, FontId};

egui_phosphor::subset! {
    pub mod icons {
        use regular::{
            GEAR, HOUSE, USER, TRASH, HEART, BELL, CHECK, FOLDER, WARNING, CALENDAR,
            ARROW_DOWN, MAGNIFYING_GLASS, ENVELOPE, LOCK, STAR,
        };
    }
}

const NAMED: &[(&str, &str)] = &[
    ("GEAR", icons::regular::GEAR),
    ("HOUSE", icons::regular::HOUSE),
    ("USER", icons::regular::USER),
    ("TRASH", icons::regular::TRASH),
    ("HEART", icons::regular::HEART),
    ("BELL", icons::regular::BELL),
    ("CHECK", icons::regular::CHECK),
    ("FOLDER", icons::regular::FOLDER),
    ("WARNING", icons::regular::WARNING),
    ("CALENDAR", icons::regular::CALENDAR),
    ("ARROW_DOWN", icons::regular::ARROW_DOWN),
    ("MAGNIFYING_GLASS", icons::regular::MAGNIFYING_GLASS),
    ("ENVELOPE", icons::regular::ENVELOPE),
    ("LOCK", icons::regular::LOCK),
    ("STAR", icons::regular::STAR),
];

const SIZES: &[f32] = &[12.0, 14.0, 16.0, 20.0, 32.0, 64.0];

fn ctx_for(bytes: &'static [u8]) -> egui::Context {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("f".to_owned(), FontData::from_static(bytes).into());
    fonts
        .families
        .insert(FontFamily::Name("f".into()), vec!["f".to_owned()]);
    let ctx = egui::Context::default();
    ctx.set_fonts(fonts);
    let _ = ctx.run_ui(Default::default(), |_| {});
    ctx
}

/// Rasterise `s` and return its advance, alpha bitmap dimensions and total ink.
fn raster(ctx: &egui::Context, s: &str, size: f32) -> Option<(f32, [usize; 2], u64)> {
    let id = FontId::new(size, FontFamily::Name("f".into()));
    let galley = ctx.fonts_mut(|f| f.layout_no_wrap(s.to_owned(), id, egui::Color32::WHITE));
    let row = galley.rows.first()?;
    let glyph = row.glyphs.first()?;
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
    Some((glyph.advance_width, [x1 - x0, y1 - y0], ink))
}

#[test]
fn subset_is_substantially_smaller() {
    let full = egui_phosphor::Variant::Regular.font_bytes().len();
    assert!(
        icons::regular::FONT.len() * 10 < full,
        "subset ({} bytes) should be far smaller than the full font ({full} bytes)",
        icons::regular::FONT.len()
    );
}

#[test]
fn every_requested_icon_renders() {
    let sub = ctx_for(&icons::regular::FONT);
    for (name, s) in NAMED {
        for &size in SIZES {
            assert!(
                raster(&sub, s, size).is_some(),
                "{name} did not rasterise at {size}px"
            );
        }
    }
}

#[test]
fn rendering_is_identical_to_the_full_font() {
    let full = ctx_for(egui_phosphor::Variant::Regular.font_bytes());
    let sub = ctx_for(&icons::regular::FONT);

    for (name, s) in NAMED {
        for &size in SIZES {
            let a = raster(&full, s, size).expect("full font must rasterise");
            let b = raster(&sub, s, size).expect("subset must rasterise");
            assert_eq!(
                a, b,
                "{name} at {size}px differs: full font {a:?} vs subset {b:?}. \
                 If only the subset changed, check that GSUB ligatures for kept \
                 glyphs (and the Latin letters they start from) are still emitted."
            );
        }
    }
}

#[test]
fn unrequested_icons_are_absent() {
    let full = ctx_for(egui_phosphor::Variant::Regular.font_bytes());
    let sub = ctx_for(&icons::regular::FONT);

    // Not listed in the `subset!` invocation above.
    for s in [
        egui_phosphor::regular::AIRPLANE,
        egui_phosphor::regular::ACORN,
        egui_phosphor::regular::USER_CIRCLE,
    ] {
        assert!(
            raster(&full, s, 32.0).is_some(),
            "control: icon should exist in the full font"
        );
        assert!(
            raster(&sub, s, 32.0).is_none(),
            "icon U+{:04X} was not requested but is present in the subset",
            s.chars().next().unwrap() as u32
        );
    }
}
