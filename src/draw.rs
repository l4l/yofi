use std::f32::consts;

use oneshot::Sender;
pub use raqote::Point;
use raqote::{DrawOptions, PathBuilder, Source, StrokeStyle};

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
    border_color: Color,
    border_width: f32,
}

impl RoundedRect {
    fn new(radius: Radius, color: Color, border_color: Color, border_width: f32) -> Self {
        Self {
            radius,
            color,
            border_color,
            border_width,
        }
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

        dt.stroke(
            &path,
            &Source::Solid(self.border_color.as_source()),
            &StrokeStyle {
                width: self.border_width,
                ..Default::default()
            },
            &DrawOptions::new(),
        );
        space
    }
}
