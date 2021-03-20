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
# if unset, places window at the center
# window_offsets = [500, -50] # in format [top_offset, left_offset]
# font = "DejaVu Sans"
font_size = 24
bg_color = 0x272822ee # ~~colors are specified in 0xRRGGBBAA format
# font_color = 0xf8f8f2ff
# HiDPI scaling factor; default is requested from compositor but
# fractional values are truncated, thus need to set it explicitly.
scale = 3

# ~~Block for input field
[input_text]
# font = ...
font_color = 0xf8f8f2ff
bg_color = 0x75715eff
# Margin/padding values are specified as in CSS
# i.e. either a signle for all directions
# or two values, the first for top/bottom and the second for left/right
# or finally four values for top, right, bottom and left directions.
margin = "5"
padding = "1.7 -4"

# ~~Block for a list with search results
[list_items]
# font = ...
font_color = 0xf8f8f2ff
selected_font_color = 0xa6e22eff
# if specified, search match will be emphasize with this color
match_color = 0xe69f66ff
margin = "5 10"
# Additional spacing between list items.
# By default there's around 10 pixels spaced,
# the amount can be reduced by specifying a negative value
item_spacing = 2
# Spacing between an icon and a text.
icon_spacing = 5

# When section presents, icons are displayed
[icon]
size = 16 # no scaling is performed, so need to choose exact size
theme = "Adwaita"
# if no icon found for an app, this on will be used instead
fallback_icon_path = "/usr/share/icons/Adwaita/16x16/categories/applications-engineering-symbolic.symbolic.png"
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

|         Key         |     Alternative     |                   Binding                    |
|---------------------|---------------------|----------------------------------------------|
| Esc                 | Ctrl + c            | Close menu                                   |
| Up Arrow            | Ctrl + k            | Select previous item                         |
| Down Arrow          | Ctrl + j            | Select next item                             |
| Return              | N/A                 | Execute selected item                        |
| Ctrl + ]            | Ctrl + ]            | Clear input                                  |
| Ctrl + w            | Ctrl + backspace    | Delete single word                           |
