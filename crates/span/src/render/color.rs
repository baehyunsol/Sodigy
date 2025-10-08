#[derive(Clone, Copy, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    None,
}

impl Color {
    pub fn render_fg(&self, s: &str) -> String {
        match self {
            Color::None => s.to_string(),
            _ => format!("\x1b[{}m{s}\x1b[0m", self.fg()),
        }
    }

    pub fn fg(&self) -> u16 {
        match self {
            Color::Red => 31,
            Color::Green => 32,
            Color::Blue => 34,
            Color::Yellow => 33,
            Color::None => 0,
        }
    }
}
