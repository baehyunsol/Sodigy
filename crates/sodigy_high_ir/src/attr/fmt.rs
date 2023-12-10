use super::{Attribute, Decorator};
use std::fmt;

impl fmt::Display for Attribute {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt, "{}",
            match self {
                Attribute::DocComment(d) => todo!(),
                Attribute::Decorator(d) => d.to_string(),
            },
        )
    }
}

impl fmt::Display for Decorator {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt, "@{}{}",
            self.name.iter().map(|i| i.id().to_string()).collect::<Vec<String>>().join("."),
            if let Some(args) = &self.args {
                format!(
                    "({})",
                    args.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "),
                )
            } else {
                String::new()
            },
        )
    }
}
