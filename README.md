# yofi

![ci_badge](https://github.com/l4l/yofi/workflows/CI/badge.svg?branch=master)

.. is a minimalistic menu for Wayland-based compositors.

**The project still in the early development stage, use with caution.**

## Configuring

Sample configuration:

```toml
# This file should be placed at ~/.config/yofi/yofi.config

# ~~Global values, used as fallback if needed
width = 400
height = 512
# font = "DejaVu Sans"
bg_color = 0x272822ee # ~~colors are specified in 0xRRGGBBAA format
# font_color = 0xf8f8f2ff

# ~~Block for input field
[input_text]
# font = ...
font_color = 0xf8f8f2ff
bg_color = 0x75715eff

# ~~Block for a list with search results
[list_items]
# font = ...
font_color = 0xf8f8f2ff
selected_font_color = 0xa6e22eff
```

## Running

For building the project you need rust compiler and cargo package manager
(usually distributed via [rustup](https://rustup.rs/)). Once installed, for
launch, you may build & run project with the following command:

```bash
cargo run --release
```

## Hotkeys

These cannot be configured yet, so the following keys are handled:

- **Esc** — closes the menu;
- **Up**/**Down** — select item at the menu;
- **Return** — launches selected app;
- **Ctrl+]** — clears the input.
