use std::f32::consts;

use oneshot::Sender;
pub use raqote::Point;
use raqote::{DrawOptions, Path, PathBuilder, Source, StrokeStyle};

pub use background::Params as BgParams;
pub use input_text::Params as InputTextParams;
pub use list_view::{ListItem, Params as ListParams};

use crate::{style::Radius, Color};

pub type DrawTarget<'a> = raqote::DrawTarget<&'a mut [u32]>;

mod background;
mod input_text;
mod list_view;

#[derive(Clone, Copy)]
pub struct Space {
    pub width: f32,
    pub height: f32,
}

pub trait Drawable {
    // Draws object to `dt` starting at `start_point` point with available `space`
    // returns used space of that object.
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, start_point: Point) -> Space;
}

pub enum Widget<'a, It = std::iter::Empty<ListItem<'a>>> {
    InputText(Box<input_text::InputText<'a>>),
    ListView(list_view::ListView<'a, It>),
    Background(background::Background),
}

pub struct Drawables<'a> {
    counter: u32,
    tx: Option<oneshot::Sender<usize>>,
    rx: Option<oneshot::Receiver<usize>>,
    state: &'a mut crate::state::State,
    background_config: BgParams,
    input_config: InputTextParams<'a>,
    list_config: ListParams,
}

impl<'a> Drawables<'a> {
    pub fn borrowed_next(&mut self) -> Option<impl Drawable + '_> {
        self.counter += 1;
        Some(match self.counter {
            1 => Widget::background(&self.background_config),
            2 => Widget::input_text(self.state.raw_input(), &self.input_config),
            3 => Widget::list_view(
                self.state.processed_entries(),
                self.state.skip_offset(),
                self.state.selected_item(),
                self.tx.take().unwrap(),
                &self.list_config,
            ),
            4 => {
                self.state
                    .update_skip_offset(self.rx.take().unwrap().recv().unwrap());
                return None;
            }
            _ => return None,
        })
    }
}

pub fn make_drawables<'c: 'it, 's: 'it, 'it>(
    config: &'c crate::config::Config,
    state: &'s mut crate::state::State,
) -> Drawables<'it> {
    let background_config = config.param();
    let input_config = config.param();
    let list_config = config.param();

    state.process_entries();

    let (tx, rx) = oneshot::channel();
    Drawables {
        counter: 0,
        tx: Some(tx),
        rx: Some(rx),
        state,

        background_config,
        input_config,
        list_config,
    }
}

impl<'a, It> Widget<'a, It> {
    pub fn input_text(text: &'a str, params: &'a InputTextParams<'a>) -> Self {
        Self::InputText(Box::new(input_text::InputText::new(text, params)))
    }

    pub fn list_view(
        items: It,
        skip_offset: usize,
        selected_item: usize,
        tx: Sender<usize>,
        params: &'a ListParams,
    ) -> Self {
        Self::ListView(list_view::ListView::new(
            items,
            skip_offset,
            selected_item,
            tx,
            params,
        ))
    }

    pub fn background(params: &'a BgParams) -> Self {
        Self::Background(background::Background::new(params))
    }
}

impl<'a, It> Drawable for Widget<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, start_point: Point) -> Space {
        match self {
            Self::InputText(w) => w.draw(dt, scale, space, start_point),
            Self::ListView(w) => w.draw(dt, scale, space, start_point),
            Self::Background(w) => w.draw(dt, scale, space, start_point),
        }
    }
}

pub struct RoundedRect {
    radius: Radius,
    color: Color,
    border: Option<Border>,
}

impl RoundedRect {
    fn new(radius: Radius, color: Color) -> Self {
        Self {
            radius,
            color,
            border: None,
        }
    }

    fn with_border(self, border: Option<Border>) -> Self {
        Self { border, ..self }
    }
}

impl Drawable for RoundedRect {
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, start_point: Point) -> Space {
        let Point { x, y, _unit } = start_point;
        let Space { width, height } = space;

        // We don't want the corner curves to overlap and thus cap the radius
        // to at most 50% of the smaller side
        let max_radius = width.min(height) / 2.0;
        let radius = &self.radius * f32::from(scale);
        let Radius {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        } = radius.min(Radius::all(max_radius));

        let mut pb = PathBuilder::new();
        pb.move_to(x, y + top_left);

        pb.arc(
            x + top_left,
            y + top_left,
            top_left,
            consts::PI,
            consts::FRAC_PI_2,
        );
        pb.arc(
            x + width - top_right,
            y + top_right,
            top_right,
            3.0 * consts::FRAC_PI_2,
            consts::FRAC_PI_2,
        );
        pb.arc(
            x + width - bottom_right,
            y + height - bottom_right,
            bottom_right,
            2.0 * consts::PI,
            consts::FRAC_PI_2,
        );
        pb.arc(
            x + bottom_left,
            y + height - bottom_left,
            bottom_left,
            consts::FRAC_PI_2,
            consts::FRAC_PI_2,
        );
        pb.line_to(x, y + top_left);
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(self.color.as_source()),
            &DrawOptions::new(),
        );

        if let Some(b) = self.border {
            b.add_stroke(dt, &path);
        }

        space
    }
}

pub struct Border {
    border_color: Color,
    border_width: f32,
}

impl Border {
    pub fn new(border_color: Color, border_width: f32) -> Self {
        Self {
            border_color,
            border_width,
        }
    }

    fn add_stroke(self, dt: &mut DrawTarget<'_>, path: &Path) {
        dt.stroke(
            path,
            &Source::Solid(self.border_color.as_source()),
            &StrokeStyle {
                width: self.border_width,
                ..Default::default()
            },
            &DrawOptions::new(),
        );
    }
}
