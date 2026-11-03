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
        format!("big {self:?}").to_ascii_lowercase()
    }

    pub fn small_flag(&self) -> String {
        format!("small {self:?}").to_ascii_lowercase()
    }
}

