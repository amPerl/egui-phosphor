# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "requests",
# ]
# ///
# This script downloads the latest version of the Phosphor Icons from the
# official website, extracts the TTF files into the `res` directory, and
# generates the rust code for each variant file.

import io
import json
import requests
import zipfile

ICON_URL = "https://phosphoricons.com/assets/phosphor-icons.zip"

# Download the latest version of the Phosphor Icons
print(f"[*] Downloading Icons ({ICON_URL})")
response = requests.get(ICON_URL)
print(f"[*] Downloaded {len(response.content)} bytes")

# Extract the TTF files and the code point to icon mappings from the
# corresponding selection.json files
variants = {}
zip = zipfile.ZipFile(io.BytesIO(response.content))

for info in zip.infolist():
    if info.filename.endswith(".ttf"):
        variant = info.filename.split("/")[1]
        font = zip.read(info.filename)
        info = json.loads(zip.read(f"Fonts/{variant}/selection.json"))

        icons = {}
        for icon in info["icons"]:
            names = icon["properties"]["name"].split(", ")
            for name in names:
                icons[name] = icon["properties"]["code"]

        variants[variant] = (font, icons)

# Remove duotone variants as they don't seem to be supported by egui font
# rendering
if "duotone" in variants:
    del variants["duotone"]

print(f"[*] Found {len(variants)} variants ({', '.join(variants.keys())})")

print("[*] Writing font and source files")
for variant, (font, icons) in variants.items():
    font_file = (
        f"res/Phosphor-{variant[0].upper() + variant[1:]}.ttf"
        if variant != "regular"
        else "res/Phosphor.ttf"
    )
    with open(font_file, "wb") as file:
        file.write(font)

# Normalise each variant's icons to name -> codepoint, dropping the variant
# suffix that Phosphor appends to non-regular names.
tables = {}
for variant, (_, icons) in variants.items():
    table = {}
    for name, code in icons.items():
        name = name.replace("-", "_").upper()
        if variant != "regular":
            name = name[: -(len(variant) + 1)]
        table[name] = code
    tables[variant] = table

# All variants have always used the same codepoints, so src/variants/ ships one
# shared table that each variant module re-exports. Verify that still holds --
# if it ever stops, the modules need their own tables again.
reference = tables["regular"]
for variant, table in tables.items():
    if table != reference:
        differing = sorted(
            name
            for name in set(table) | set(reference)
            if table.get(name) != reference.get(name)
        )
        raise SystemExit(
            f"[!] {variant} no longer shares regular's codepoints "
            f"({len(differing)} differ, e.g. {differing[:5]}).\n"
            f"    src/variants/ assumes one shared table; give each variant its "
            f"own again before regenerating."
        )
print(f"[*] All {len(tables)} variants agree on {len(reference)} codepoints")

with open("src/variants/codepoints.rs", "w", newline="\n") as file:
    file.write("#![allow(unused)]\n")
    for name, code in reference.items():
        file.write(f'pub const {name}: &str = "\\u{{{hex(code)[2:].upper()}}}";\n')
    file.write("\npub const ICONS: &[(&str, &str)] = &[\n    ")
    file.write(",\n    ".join(f'("{name}", {name})' for name in reference))
    file.write(",\n];\n")

for variant in tables:
    with open(f"src/variants/{variant}.rs", "w", newline="\n") as file:
        file.write("pub use super::codepoints::*;\n")

print("[*] Done!")
