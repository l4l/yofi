# yofi

![ci_badge](https://github.com/l4l/yofi/workflows/CI/badge.svg?branch=master)

.. is a minimalistic menu for Wayland-based compositors.

**The project still in the early development stage, use with caution.**

## Configuring

TBD

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
