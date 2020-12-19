pub use raqote::{DrawTarget, Point};

pub use background::Params as BgParams;
pub use input_text::Params as InputTextParams;
pub use list_view::{ListItem, Params as ListParams};

mod background;
mod input_text;
mod list_view;

const FONT_SIZE: f32 = 24.0;

#[derive(Clone, Copy)]
pub struct Space {
    pub width: f32,
    pub height: f32,
}

pub trait Drawable {
    // Draws object to `dt` starting at `start_point` point with availabpe `space`
    // returns used space of that object.
    fn draw(self, dt: &mut DrawTarget, space: Space, start_point: Point) -> Space;
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

    pub fn list_view(items: It, selected_item: usize, params: ListParams) -> Self {
        Self::ListView(list_view::ListView::new(items, selected_item, params))
    }

    pub fn background(params: BgParams) -> Self {
        Self::Background(background::Background::new(params))
    }
}

impl<'a, It> Drawable for Widget<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, dt: &mut DrawTarget, space: Space, start_point: Point) -> Space {
        match self {
            Self::InputText(w) => w.draw(dt, space, start_point),
            Self::ListView(w) => w.draw(dt, space, start_point),
            Self::Background(w) => w.draw(dt, space, start_point),
        }
    }
}
