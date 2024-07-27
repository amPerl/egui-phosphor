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

    with open(f"src/variants/{variant}.rs", "w") as file:
        file.write("#![allow(unused)]\n")
        for name, code in icons.items():
            code_point = hex(code)[2:].upper()

            name = name.replace("-", "_").upper()
            if variant != "regular":
                name = name[: -(len(variant) + 1)]

            file.write(f'pub const {name}: &str = "\\u{{{code_point}}}";\n')

print("[*] Done!")
