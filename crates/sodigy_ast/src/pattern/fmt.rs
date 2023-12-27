use super::{Pattern, PatternKind};
use sodigy_error::RenderError;
use std::fmt;

impl fmt::Display for Pattern {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}{}{}",
            if let Some(bind) = &self.bind {
                // `$y @ $y` is just `$y`
                if matches!(self.kind, PatternKind::Binding(_)) {
                    String::new()
                }

                else {
                    format!("${} @ ", bind.id())
                }
            } else {
                String::new()
            },
            if self.kind.needs_paren() && (self.bind.is_some() || self.ty.is_some()) {
                format!("({})", self.kind)
            }
            else {
                self.kind.to_string()
            },
            if let Some(ty) = &self.ty {
                format!(": {ty}")
            } else {
                String::new()
            },
        )
    }
}

impl fmt::Display for PatternKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            PatternKind::Identifier(_)
            | PatternKind::Binding(_)
            | PatternKind::Char(_)
            | PatternKind::Wildcard
            | PatternKind::Shorthand
            | PatternKind::Number { .. } => self.render_error(),
            PatternKind::Range {
                from, to, inclusive
            } => format!(
                "{}{}{}",
                if let Some(p) = from {
                    if p.needs_paren() {
                        format!("({p})")
                    } else {
                        p.to_string()
                    }
                }
                else {
                    String::new()
                },
                if *inclusive { "..~" } else { ".." },
                if let Some(p) = to {
                    if p.needs_paren() {
                        format!("({p})")
                    } else {
                        p.to_string()
                    }
                }
                else {
                    String::new()
                },
            ),
            p @ (PatternKind::Tuple(patterns)
            | PatternKind::List(patterns)) => {
                let is_tuple = matches!(p, PatternKind::Tuple(_));
                let (start, end) = if is_tuple {
                    ('(', ')')
                } else {
                    ('[', ']')
                };

                format!(
                    "{start}{}{end}",
                    patterns.iter().map(
                        |pat| pat.to_string()
                    ).collect::<Vec<String>>().join(", ")
                )
            },
            PatternKind::Path(names) => names.iter().map(
                |name| name.id().to_string()
            ).collect::<Vec<String>>().join("."),
            PatternKind::Or(lhs, rhs) => format!(
                "{} | {}",
                if lhs.needs_paren() {
                    format!("({lhs})")
                } else {
                    lhs.to_string()
                },
                if rhs.needs_paren() {
                    format!("({rhs})")
                } else {
                    rhs.to_string()
                },
            ),
            PatternKind::TupleStruct {
                name, fields,
            } => {
                let name = name.iter().map(
                    |name| name.id().to_string()
                ).collect::<Vec<String>>().join(".");
                let patterns = fields.iter().map(
                    |pat| pat.to_string()
                ).collect::<Vec<String>>().join(", ");

                format!("{name}({patterns})")
            },
            _ => todo!(),
        };

        write!(fmt, "{s}")
    }
}

impl RenderError for PatternKind {
    fn render_error(&self) -> String {
        match self {
            PatternKind::Identifier(id) => id.render_error(),
            PatternKind::Binding(id) => format!("${}", id.render_error()),
            PatternKind::Char(c) => format!("{c:?}"),
            PatternKind::Wildcard => String::from("_"),
            PatternKind::Shorthand => String::from(".."),
            PatternKind::Number { num, is_negative } => format!(
                "{}{num}",
                if *is_negative { "-" } else { "" },
            ),
            PatternKind::Range {
                from, to, inclusive,
            } => format!(
                "{}{}{}",
                if let Some(p) = from { p.kind.render_error() } else { String::new() },
                if *inclusive { "..~" } else { ".." },
                if let Some(p) = to { p.kind.render_error() } else { String::new() },
            ),
            p @ (PatternKind::Tuple(patterns)
            | PatternKind::List(patterns)) => {
                let is_tuple = matches!(p, PatternKind::Tuple(_));
                let (start, end) = if is_tuple {
                    ('(', ')')
                } else {
                    ('[', ']')
                };

                format!(
                    "{start}{}{end}",
                    if patterns.len() > 4 {
                        String::from("...")
                    } else {
                        patterns.iter().map(
                            |pat| pat.kind.render_error()
                        ).collect::<Vec<String>>().join(", ")
                    },
                )
            },
            p @ (PatternKind::Path(path)
            | PatternKind::Struct { struct_name: path, .. }
            | PatternKind::TupleStruct { name: path, .. }) => {
                let name = path.iter().map(
                    |p| p.id().render_error()
                ).collect::<Vec<String>>().join(".");

                format!(
                    "{name}{}",
                    match p {
                        PatternKind::Path(_) => "",
                        PatternKind::Struct { .. } => "{...}",
                        PatternKind::TupleStruct { .. } => "(...)",
                        _ => unreachable!(),
                    },
                )
            },
            PatternKind::Or(p1, p2) => format!(
                "{} | {}",
                p1.kind.render_error(),
                p2.kind.render_error(),
            ),
        }
    }
}

impl Pattern {
    pub fn needs_paren(&self) -> bool {
        self.kind.needs_paren()
        || self.ty.is_some()
        || self.bind.is_some()
    }
}

impl PatternKind {
    pub fn needs_paren(&self) -> bool {
        // Do not use wildcards
        match self {
            PatternKind::Identifier(_)
            | PatternKind::Binding(_)
            | PatternKind::Char(_)
            | PatternKind::Wildcard
            | PatternKind::Shorthand
            | PatternKind::Number { .. }
            | PatternKind::Tuple(_)
            | PatternKind::List(_)
            | PatternKind::Path(_)
            | PatternKind::Struct { .. }
            | PatternKind::TupleStruct { .. } => false,
            PatternKind::Range { .. }
            | PatternKind::Or(_, _) => true,
        }
    }
}
