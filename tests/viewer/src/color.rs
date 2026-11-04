use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    pub fn big_flag(&self) -> String {
        format!("big-circle {self:?}-bg").to_ascii_lowercase()
    }

    pub fn small_flag(&self) -> String {
        format!("small-circle {self:?}-bg").to_ascii_lowercase()
    }
}

