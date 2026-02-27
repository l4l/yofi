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

pub fn render_to_buffer(
    config: &config::Config,
    state: &mut state::State,
    scale: u16,
    width: i32,
    height: i32,
    buffer: &mut [u32],
) {
    use draw::Drawable;

    let mut dt = DrawTarget::from_backing(width, height, buffer);
    let mut space_left = draw::Space {
        width: width as f32,
        height: height as f32,
    };
    let mut point = draw::Point::new(0., 0.);

    let (mut drawables, dyn_space) = draw::make_drawables(config, state, scale);
    if let Some(dyn_space) = dyn_space {
        space_left.height = space_left.height.min(dyn_space.height);
    }
    while let Some(d) = drawables.borrowed_next() {
        let occupied = d.draw(&mut dt, scale, space_left, point);
        point.y += occupied.height;
        space_left.height -= occupied.height;
    }
}
