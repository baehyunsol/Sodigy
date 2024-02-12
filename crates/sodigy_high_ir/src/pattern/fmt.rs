use super::{NumberLike, Pattern, PatternKind};
use sodigy_error::RenderError;
use std::fmt;

impl fmt::Display for Pattern {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let result = format!(
            "{}{}{}",
            if let Some(name) = self.bind {
                // `$x @ $x` is just `$x`
                if matches!(self.kind, PatternKind::Binding(_)) {
                    String::new()
                }

                else {
                    format!("${} @ ", name.id())
                }
            } else {
                String::new()
            },
            self.kind,
            if let Some(ty) = &self.ty {
                format!(": {ty}")
            } else {
                String::new()
            },
        );

        write!(fmt, "{result}")
    }
}

impl fmt::Display for PatternKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let result = match self {
            PatternKind::Binding(name) => format!("${name}"),
            PatternKind::TupleStruct { name, fields } => format!(
                "{}({})",
                name.iter().map(
                    |name| name.id().to_string()
                ).collect::<Vec<_>>().join("."),
                fields.iter().map(
                    |pat| pat.to_string()
                ).collect::<Vec<_>>().join(", ")
            ),
            PatternKind::Wildcard => String::from("_"),
            _ => todo!(),
        };

        write!(fmt, "{result}")
    }
}

impl RenderError for NumberLike {
    fn render_error(&self) -> String {
        match self {
            NumberLike::OpenEnd { .. } => todo!(),  // Do we even need this branch?
            NumberLike::Exact(num) => num.to_string(),
            NumberLike::MinusEpsilon { .. } => todo!(),  // Do we even need this branch?
        }
    }
}
