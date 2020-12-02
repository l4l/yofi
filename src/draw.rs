pub use raqote::{DrawTarget, Point};

pub use list_view::ListItem;

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
    fn draw(self, dt: &mut DrawTarget, space: Space, start_point: Point) -> Space;
}

pub enum Widget<'a, It> {
    InputText(input_text::InputText<'a>),
    ListView(list_view::ListView<'a, It>),
}

impl<'a, It> Widget<'a, It> {
    pub fn input_text(text: &'a str) -> Self {
        Self::InputText(input_text::InputText::new(text))
    }

    pub fn list_view(items: It, selected_item: usize) -> Self {
        Self::ListView(list_view::ListView::new(items, selected_item))
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
        }
    }
}
