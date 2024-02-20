use super::{ArgDef, TypeDef};
use std::fmt;

impl fmt::Display for TypeDef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.as_expr())
    }
}

impl fmt::Display for ArgDef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = Vec::with_capacity(self.attributes.len() + 1);

        for attr in self.attributes.iter() {
            result.push(attr.to_string());
        }

        result.push(format!(
            "{}{}{}",
            self.name.id(),
            if self.has_question_mark {
                String::from("?")
            } else {
                String::new()
            },
            if let Some(ty) = &self.ty {
                format!(": {ty}")
            } else {
                String::new()
            },
        ));

        write!(fmt, "{}", result.join("\n"))
    }
}
