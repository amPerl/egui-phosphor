# egui_phosphor

Bundles [Phosphor icons](https://phosphoricons.com/) with boilerplate to use in your egui app.

## Installation

Add the crate as a dependency in Cargo.toml:
```toml
egui-phosphor = "0.1.0"
```

On startup, update the fonts in your egui context:
```rust
let mut fonts = egui::FontDefinitions::default();
egui_phosphor::add_to_fonts(&mut fonts);

cc.egui_ctx.set_fonts(fonts);
```

## Usage

Use the constants provided by the crate in your text:
```rust
ui.label(egui::RichText::new(format!("FILE_CODE {}", egui_phosphor::FILE_CODE)).size(32.0));
```

## License

egui-phosphor is licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE). Phosphor Icons are licensed under [MIT](https://github.com/phosphor-icons/web/blob/master/LICENSE).