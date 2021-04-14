use font_kit::loaders::freetype::Font;
use oneshot::Sender;
use raqote::{DrawOptions, Source};
pub use raqote::{DrawTarget, Point};

pub use background::Params as BgParams;
pub use input_text::Params as InputTextParams;
pub use list_view::{ListItem, Params as ListParams};

mod background;
mod input_text;
mod list_view;

#[derive(Clone, Copy)]
pub struct Space {
    pub width: f32,
    pub height: f32,
}

pub trait Drawable {
    // Draws object to `dt` starting at `start_point` point with availabpe `space`
    // returns used space of that object.
    fn draw(self, dt: &mut DrawTarget, scale: u16, space: Space, start_point: Point) -> Space;
}

pub enum Widget<'a, It = std::iter::Empty<ListItem<'a>>> {
    InputText(input_text::InputText<'a>),
    ListView(list_view::ListView<'a, It>),
    Background(background::Background),
}

impl<'a, It> Widget<'a, It> {
    pub fn input_text(text: &'a str, params: InputTextParams) -> Self {
        Self::InputText(input_text::InputText::new(text, params))
    }

    pub fn list_view(
        items: It,
        skip_offset: usize,
        selected_item: usize,
        tx: Sender<usize>,
        params: ListParams,
    ) -> Self {
        Self::ListView(list_view::ListView::new(
            items,
            skip_offset,
            selected_item,
            tx,
            params,
        ))
    }

    pub fn background(params: BgParams) -> Self {
        Self::Background(background::Background::new(params))
    }
}

impl<'a, It> Drawable for Widget<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, dt: &mut DrawTarget, scale: u16, space: Space, start_point: Point) -> Space {
        match self {
            Self::InputText(w) => w.draw(dt, scale, space, start_point),
            Self::ListView(w) => w.draw(dt, scale, space, start_point),
            Self::Background(w) => w.draw(dt, scale, space, start_point),
        }
    }
}

fn draw_text(
    dt: &mut DrawTarget,
    text: &str,
    font: &Font,
    point_size: f32,
    start: Point,
    source: Source,
    opts: &DrawOptions,
) {
    let start = pathfinder_geometry::vector::vec2f(start.x, start.y);
    let (_, ids, positions) = text
        .chars()
        .filter_map(|c| {
            let id = font.glyph_for_char(c);

            let id = if let Some(id) = id {
                id
            } else {
                log::warn!("cannot find glyph for {:?}", c);
                return None;
            };

            let advance = match font.advance(id) {
                Ok(x) => x,
                Err(err) => {
                    log::warn!("cannot advance font for {:?}: {}", c, err);
                    return None;
                }
            };

            Some((id, advance))
        })
        .fold(
            (start, vec![], vec![]),
            |(start, mut ids, mut positions), (id, advance)| {
                ids.push(id);
                positions.push(Point::new(start.x(), start.y()));

                let delta = advance * point_size / 24. / 96.;

                (start + delta, ids, positions)
            },
        );

    dt.draw_glyphs(font, point_size, &ids, &positions, &source, &opts);
}
