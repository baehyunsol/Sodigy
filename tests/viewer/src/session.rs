use crate::{Color, Id};
use std::collections::HashMap;

pub struct Session {
    // These data have to be collected before any nodes are rendered.
    pub title: HashMap<Id, (String, Option<Color>)>,
    pub children: HashMap<Id, Option<Vec<(String, Id)>>>,
    pub parent: HashMap<Id, Vec<(Id, usize)>>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            title: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
        }
    }
}

