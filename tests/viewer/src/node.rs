use crate::{Color, Graphic, Id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: Id,
    pub title: String,
    pub flag: Option<Color>,
    pub metadata: Option<Vec<Graphic>>,
    pub children: Option<Vec<(String, Id)>>,
}
