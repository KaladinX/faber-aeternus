pub use iocraft::prelude::*;

// Catppuccin Macchiato palette
pub struct Theme {
    pub surface0: Color,
    pub surface1: Color,
    pub text: Color,
    pub subtext0: Color,
    pub primary: Color,
    pub secondary: Color,
    pub error: Color,
}

pub const THEME: Theme = Theme {
    surface0: Color::Rgb { r: 54, g: 58, b: 79 },
    surface1: Color::Rgb { r: 73, g: 77, b: 100 },
    text: Color::Rgb { r: 202, g: 211, b: 245 },
    subtext0: Color::Rgb { r: 165, g: 173, b: 203 },
    primary: Color::Rgb { r: 138, g: 173, b: 244 },
    secondary: Color::Rgb { r: 245, g: 169, b: 127 },
    error: Color::Rgb { r: 237, g: 135, b: 150 },
};
