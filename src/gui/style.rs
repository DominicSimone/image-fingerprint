use iced::{button, Background, Color, Vector};

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);
const RED: Color = Color::from_rgb(
    0xb7 as f32 / 255.0,
    0x47 as f32 / 255.0,
    0xb9 as f32 / 255.0,
);
const GREEN: Color = Color::from_rgb(68.0 / 255.0, 162.0 / 255.0, 174.0 / 255.0);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);
const HOVERED_RED: Color = Color::from_rgb(147.0 / 255.0, 57.0 / 255.0, 148.0 / 255.0);
const HOVERED_GREEN: Color = Color::from_rgb(58.0 / 255.0, 139.0 / 255.0, 149.0 / 255.0);

pub enum Button {
    Primary,
    Additive,
    Destructive,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let (background, text_color) = match self {
            Button::Primary => (Some(ACTIVE), Color::WHITE),
            Button::Destructive => (Some(RED), Color::WHITE),
            Button::Additive => (Some(GREEN), Color::WHITE),
        };

        button::Style {
            text_color,
            background: background.map(Background::Color),
            border_radius: 5.0,
            shadow_offset: Vector::new(0.0, 0.0),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        let background = match self {
            Button::Primary => Some(HOVERED),
            Button::Destructive => Some(HOVERED_RED),
            Button::Additive => Some(HOVERED_GREEN),
        };

        button::Style {
            background: background.map(Background::Color),
            ..active
        }
    }
}

