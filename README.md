# egui_phosphor

Bundles [Phosphor icons](https://phosphoricons.com/) with boilerplate to use in your egui app.

## Installation

Add the crate as a dependency in Cargo.toml:

```toml
egui-phosphor = "0.3.0"
```

On startup, update the fonts in your egui context:

```rust
let mut fonts = egui::FontDefinitions::default();
egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

cc.egui_ctx.set_fonts(fonts);
```

The above `add_to_fonts` helper adds the chosen variant (`Regular`) as a fallback to the primary egui `Proportional` font so that when you use Phosphor icons mixed with plain text in labels, the icon font will take over where necessary. If you want to add multiple variants of Phosphor icons, see [this example](examples/multiple_variants.rs) which shows all variants in use.

## Usage

Use the constants provided by the crate in your text:

```rust
ui.label(egui::RichText::new(format!("FILE_CODE {}", egui_phosphor::regular::FILE_CODE)).size(32.0));
```

**Note: Make sure to use the appropriate character codes for your chosen variant!** This means for `Variant::Regular` you should use `regular::FILE_CODE`, for `Variant::Fill` you should use `fill::FILE_CODE` etc.

## License

egui-phosphor is licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE). Phosphor Icons are licensed under [MIT](https://github.com/phosphor-icons/web/blob/master/LICENSE).
