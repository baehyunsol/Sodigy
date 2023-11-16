use super::{NumberLike, Pattern, PatternKind};
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
            _ => todo!(),
        };

        write!(fmt, "{result}")
    }
}

impl NumberLike {
    pub fn render_error(&self) -> String {
        match self {
            NumberLike::OpenEnd { .. } => todo!(),  // Do we even need this branch?
            NumberLike::Exact { num, is_negative } => format!(
                "{}{num}",
                if *is_negative { "-" } else { "" },
            ),
            NumberLike::MinusEpsilon { .. } => todo!(),  // Do we even need this branch?
        }
    }
}
