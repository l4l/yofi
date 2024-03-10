pub(crate) use color::Color;
pub(crate) use desktop::Entry as DesktopEntry;
pub(crate) use draw::DrawTarget;

mod color;
mod draw;
mod exec;
mod font;
mod icon;
mod input_parser;
mod style;
mod usage_cache;

pub mod config;
pub mod desktop;
pub mod mode;
pub mod state;
pub mod window;

#[macro_export]
macro_rules! prog_name {
    () => {
        "yofi"
    };
}
