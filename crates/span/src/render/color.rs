#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

pub fn apply_colors(line: &[u8], colors: &[Color]) -> String {
    assert_eq!(line.len(), colors.len());
    let mut result = vec![];
    let mut curr_color = Color::None;
    let mut buffer = vec![];

    for (byte, color) in line.iter().zip(colors.iter()) {
        if *color != curr_color {
            result.push(curr_color.render_fg(&String::from_utf8_lossy(&buffer)));
            curr_color = *color;
            buffer = vec![*byte];
        }

        else {
            buffer.push(*byte);
        }
    }

    if !buffer.is_empty() {
        result.push(curr_color.render_fg(&String::from_utf8_lossy(&buffer)));
    }

    result.concat()
}
