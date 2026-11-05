use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Black,
    White,
}

impl Color {
    pub fn big_flag(&self) -> String {
        format!("big-circle {self:?}-bg").to_ascii_lowercase()
    }

    pub fn small_flag(&self) -> String {
        format!("small-circle {self:?}-bg").to_ascii_lowercase()
    }

    pub fn all_filters() -> Vec<Option<Color>> {
        vec![
            None,
            Some(Color::Red),
            Some(Color::Green),
            Some(Color::Blue),
            Some(Color::Yellow),
            Some(Color::Black),
            Some(Color::White),
        ]
    }
}

