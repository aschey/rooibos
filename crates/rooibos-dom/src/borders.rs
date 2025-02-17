use ratatui::style::{Color, Style, Styled};
use ratatui::symbols::border;
use ratatui::widgets::block::Title;
use taffy::LengthPercentage;

#[derive(Clone, Copy, Debug, Default)]
pub enum BorderType {
    #[default]
    Solid,
    Round,
    Double,
    Thick,
    Outer,
    Inner,
    Wide,
    Tall,
    ThickWide,
    ThickTall,
    ThickFull,
    Empty,
}

impl From<BorderType> for border::Set {
    fn from(value: BorderType) -> Self {
        match value {
            BorderType::Solid => border::PLAIN,
            BorderType::Round => border::ROUNDED,
            BorderType::Double => border::DOUBLE,
            BorderType::Thick => border::THICK,
            BorderType::Outer => border::QUADRANT_OUTSIDE,
            BorderType::Inner => border::QUADRANT_INSIDE,
            BorderType::Wide => border::ONE_EIGHTH_WIDE,
            BorderType::Tall => border::ONE_EIGHTH_TALL,
            BorderType::ThickWide => border::PROPORTIONAL_WIDE,
            BorderType::ThickTall => border::PROPORTIONAL_TALL,
            BorderType::ThickFull => border::FULL,
            BorderType::Empty => border::EMPTY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Borders {
    borders: ratatui::widgets::Borders,
    border_type: BorderType,
    titles: Vec<Title<'static>>,
    style: Style,
}

impl Default for Borders {
    fn default() -> Self {
        Self {
            borders: ratatui::widgets::Borders::default(),
            border_type: BorderType::default(),
            titles: Vec::new(),
            style: Style::default().fg(Color::Reset),
        }
    }
}

impl Styled for Borders {
    type Item = Self;
    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl Borders {
    pub fn all() -> Self {
        Self {
            borders: ratatui::widgets::Borders::ALL,
            ..Default::default()
        }
    }

    pub fn top() -> Self {
        Self {
            borders: ratatui::widgets::Borders::TOP,
            ..Default::default()
        }
    }

    pub fn bottom() -> Self {
        Self {
            borders: ratatui::widgets::Borders::BOTTOM,
            ..Default::default()
        }
    }

    pub fn left() -> Self {
        Self {
            borders: ratatui::widgets::Borders::LEFT,
            ..Default::default()
        }
    }

    pub fn right() -> Self {
        Self {
            borders: ratatui::widgets::Borders::RIGHT,
            ..Default::default()
        }
    }

    pub fn x() -> Self {
        Self {
            borders: ratatui::widgets::Borders::LEFT | ratatui::widgets::Borders::RIGHT,
            ..Default::default()
        }
    }

    pub fn y() -> Self {
        Self {
            borders: ratatui::widgets::Borders::TOP | ratatui::widgets::Borders::BOTTOM,
            ..Default::default()
        }
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    pub fn solid(self) -> Self {
        self.border_type(BorderType::Solid)
    }

    pub fn round(self) -> Self {
        self.border_type(BorderType::Round)
    }

    pub fn double(self) -> Self {
        self.border_type(BorderType::Double)
    }

    pub fn thick(self) -> Self {
        self.border_type(BorderType::Thick)
    }

    pub fn outer(self) -> Self {
        self.border_type(BorderType::Outer)
    }

    pub fn inner(self) -> Self {
        self.border_type(BorderType::Inner)
    }

    pub fn wide(self) -> Self {
        self.border_type(BorderType::Wide)
    }

    pub fn tall(self) -> Self {
        self.border_type(BorderType::Tall)
    }

    pub fn thick_wide(self) -> Self {
        self.border_type(BorderType::ThickWide)
    }

    pub fn thick_tall(self) -> Self {
        self.border_type(BorderType::ThickTall)
    }

    pub fn thick_full(self) -> Self {
        self.border_type(BorderType::ThickFull)
    }

    pub fn empty(self) -> Self {
        self.border_type(BorderType::Empty)
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Title<'static>>,
    {
        self.titles.push(title.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn into_block(self) -> ratatui::widgets::Block<'static> {
        let mut block = ratatui::widgets::Block::new()
            .borders(self.borders)
            .border_set(self.border_type.into())
            .border_style(self.style);

        for title in self.titles {
            block = block.title(title);
        }
        block
    }

    pub fn to_rect(&self) -> taffy::Rect<LengthPercentage> {
        let set = LengthPercentage::Length(1.0);
        let mut rect = taffy::Rect::zero();
        let borders = self.borders;
        if borders.intersects(ratatui::widgets::Borders::TOP) {
            rect.top = set;
        }
        if borders.intersects(ratatui::widgets::Borders::BOTTOM) {
            rect.bottom = set;
        }
        if borders.intersects(ratatui::widgets::Borders::LEFT) {
            rect.left = set;
        }
        if borders.intersects(ratatui::widgets::Borders::RIGHT) {
            rect.right = set;
        }
        rect
    }
}
