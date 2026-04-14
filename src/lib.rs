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

pub struct Viewport {
    buf_size: (i32, i32),
    content_offset: (f32, f32),
    content_size: (f32, f32),
}

impl Viewport {
    pub fn full(width: i32, height: i32) -> Self {
        Self {
            buf_size: (width, height),
            content_offset: (0.0, 0.0),
            content_size: (width as f32, height as f32),
        }
    }

    pub fn inset(mut self, offset: (f32, f32), content: (f32, f32)) -> Self {
        self.content_offset = offset;
        self.content_size = content;
        self
    }
}

pub fn render_to_buffer(
    config: &config::Config,
    state: &mut state::State,
    scale: u16,
    buffer: &mut [u32],
    viewport: Viewport,
) {
    use draw::Drawable;

    let mut dt = DrawTarget::from_backing(viewport.buf_size.0, viewport.buf_size.1, buffer);
    let mut space_left = draw::Space {
        width: viewport.content_size.0,
        height: viewport.content_size.1,
    };
    let mut point = draw::Point::new(viewport.content_offset.0, viewport.content_offset.1);

    let (mut drawables, dyn_space) = draw::make_drawables(config, state, scale);
    if let Some(dyn_space) = dyn_space {
        space_left.height = space_left.height.min(dyn_space.height);
    }
    while let Some(d) = drawables.borrowed_next() {
        let occupied = d.draw(&mut dt, scale, space_left, point);
        debug_assert!(occupied.width <= space_left.width && occupied.height <= space_left.height);
        point.y += occupied.height;
        space_left.height -= occupied.height;
    }
}
