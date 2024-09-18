use super::{Attribute, Decorator};
use std::fmt;

impl<Expr: fmt::Display> fmt::Display for Attribute<Expr> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Attribute::DocComment(d) => {
                let doc = d.id().to_string();
                let mut lines = vec![];

                for line in doc.lines() {
                    lines.push(format!("#> {line}"));
                }

                write!(fmt, "{}", lines.join("\n"))
            },
            Attribute::Decorator(d) => write!(fmt, "{d}"),
        }
    }
}

impl<Expr: fmt::Display> fmt::Display for Decorator<Expr> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt, "@{}{}",
            self.name.iter().map(
                |name| name.id().to_string()
            ).collect::<Vec<_>>().join("."),
            if let Some(args) = &self.args {
                format!(
                    "({})",
                    args.iter().map(
                        |arg| arg.to_string()
                    ).collect::<Vec<_>>().join(", "),
                )
            } else {
                String::new()
            },
        )
    }
}
