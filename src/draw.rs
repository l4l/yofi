use oneshot::Sender;
pub use raqote::Point;

pub use background::Params as BgParams;
pub use input_text::Params as InputTextParams;
pub use list_view::{ListItem, Params as ListParams};

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
    // Draws object to `dt` starting at `start_point` point with availabpe `space`
    // returns used space of that object.
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, start_point: Point) -> Space;
}

pub enum Widget<'a, It = std::iter::Empty<ListItem<'a>>> {
    InputText(Box<input_text::InputText<'a>>),
    ListView(list_view::ListView<'a, It>),
    Background(background::Background<'a>),
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
