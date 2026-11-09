use crate::{Color, Graphic, Id};

#[derive(Clone, Debug)]
pub struct Node {
    pub id: Id,
    pub title: String,
    pub flag: Option<Color>,
    pub metadata: Option<Vec<Graphic>>,
    pub children: Option<Vec<(String, Id)>>,
}

