# yofi

![ci_badge](https://github.com/l4l/yofi/workflows/CI/badge.svg?branch=master)

.. is a minimalistic menu for Wayland-based compositors.

## Installation

Make sure you setup a wayland environment, in particularly `WAYLAND_DISPLAY`
env var must be set. `wlr_layer_shell` protocol is not necessary but preferred.
There are several installation options:

- Pre-built release binaries are published at the [Release page](https://github.com/l4l/yofi/releases).
Although these are built in Ubuntu environment it should also work for other Linux distributions.
- \[for Archlinux\] there are [yofi-bin](https://aur.archlinux.org/packages/yofi-bin/) and
[yofi-git](https://aur.archlinux.org/packages/yofi-git/) AUR packages for binary and from-source builds.
- Or you can manually [build from sources](#building).

## User documentation

User documentation is located at [Wiki pages](https://github.com/l4l/yofi/wiki).
Feel free to [open an issue](https://github.com/l4l/yofi/issues/new) if something
is unclear, missing or outdated.

## Building

For building the project you need rust compiler and cargo package manager
(usually distributed via [rustup](https://rustup.rs/)). Once installed you
may build & run the project with the following command:

```bash
cargo run --release
```

## Contributing

Contributions are welcome, but make sure that:

- \[If that's a new feature or it changes the existing behavior\] you've discussed it in the issue page before the implementation.
- Your patch is not a refactoring.
- rustfmt and clippy are checked.
- \[optionally\] Added docs if necessary and an entry in CHANGELOG.md.
