use anyhow::Context;
use reqwest::blocking;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use zip::ZipArchive;

const ICON_URL: &str = "https://phosphoricons.com/assets/phosphor-icons.zip";

fn main() -> anyhow::Result<()> {
    let content = blocking::get(ICON_URL)?.bytes()?;
    let reader = io::Cursor::new(content);
    let mut zip = ZipArchive::new(reader)?;

    let variants = extract_variants(&mut zip)?;
    save_fonts_and_generate_code(variants)?;

    println!("[*] Done!");
    Ok(())
}

fn extract_variants(
    zip: &mut ZipArchive<std::io::Cursor<bytes::Bytes>>,
) -> anyhow::Result<HashMap<std::string::String, (Vec<u8>, HashMap<std::string::String, u32>)>> {
    let mut variants: HashMap<String, (Vec<u8>, HashMap<String, u32>)> = HashMap::new();
    let mut zip_ = zip.clone();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filename = file.name();

        if filename.ends_with(".ttf") {
            println!("[*] Extracting {}", filename);
            let parts: Vec<&str> = filename.split('/').collect();
            let variant = parts[1].to_string();
            let mut font = Vec::new();
            file.read_to_end(&mut font)?;

            let mut selection_file = zip_.by_name(&format!("Fonts/{}/selection.json", variant))?;
            let mut selection_data = String::new();
            selection_file.read_to_string(&mut selection_data)?;
            let info: serde_json::Value = serde_json::from_str(&selection_data)?;

            let mut icons: HashMap<String, u32> = HashMap::new();
            for icon in info["icons"]
                .as_array()
                .context("Could not get infos as array")?
            {
                let names: Vec<String> = icon["properties"]["name"]
                    .as_str()
                    .context("Could not get name as string")
                    .context("Error getting result")?
                    .split(", ")
                    .map(String::from)
                    .collect();
                let code = icon["properties"]["code"]
                    .as_u64()
                    .context("Could not convert to u64")? as u32;

                for name in names {
                    icons.insert(name, code);
                }
            }

            variants.insert(variant, (font, icons));
        }
    }
    variants.remove("duotone");
    Ok(variants)
}

fn save_fonts_and_generate_code(
    variants: HashMap<String, (Vec<u8>, HashMap<String, u32>)>,
) -> anyhow::Result<()> {
    for (variant, (font, icons)) in variants {
        let font_file = if variant == "regular" {
            "res/Phosphor.ttf".to_string()
        } else {
            format!("res/Phosphor-{}.ttf", capitalize(&variant))
        };

        let mut font_file_handle = File::create(&font_file)?;
        font_file_handle.write_all(&font)?;

        let source_file = format!("src/variants/{}.rs", variant);
        let mut source_file_handle = File::create(&source_file)?;
        source_file_handle.write_all(b"#![allow(unused)]\n")?;

        let mut sorted_icons: Vec<_> = icons
            .into_iter()
            .map(|(name, code)| {
                let mut const_name = name.replace('-', "_").to_uppercase();
                if variant != "regular" {
                    const_name = const_name
                        .trim_end_matches(&variant.to_uppercase())
                        .trim_end_matches('_')
                        .to_string();
                }
                (const_name, format!("{:X}", code))
            })
            .collect();

        sorted_icons.sort_by(|a, b| a.0.cmp(&b.0));

        for (const_name, code_point) in sorted_icons {
            writeln!(
                source_file_handle,
                "pub const {}: &str = \"\\u{{{}}}\";",
                const_name, code_point
            )?;
        }
    }

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        _ => String::new(),
    }
}
