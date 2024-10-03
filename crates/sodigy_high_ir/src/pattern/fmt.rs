use super::{
    NumberLike,
    Pattern,
    PatternKind,
    StringPattern,
    RangeType,
};
use sodigy_error::RenderError;
use sodigy_prelude as prelude;
use std::fmt;

impl Pattern {
    // for error messages
    pub fn get_type_string(&self) -> String {
        match &self.kind {
            PatternKind::String(s) => if s.is_binary {
                prelude::BYTES.0.to_string()
            } else {
                prelude::STRING.0.to_string()
            },
            PatternKind::Tuple(_) => prelude::TUPLE.0.to_string(),
            PatternKind::List(_) => prelude::LIST.0.to_string(),
            _ => String::from("_"),  // TODO
        }
    }
}

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
            PatternKind::Constant(v) => v.to_string(),
            PatternKind::Binding(name) => format!("${name}"),
            PatternKind::String(StringPattern {
                strings,
                open_prefix,
                open_suffix,
                is_binary,
            }) => {
                let mut units = vec![];

                if *open_prefix {
                    units.push(String::new());
                }

                for s in strings.iter() {
                    units.push(format!(
                        "{}{:?}",
                        if *is_binary { "b" } else { "" },
                        s.id().to_string(),
                    ));
                }

                if *open_suffix {
                    units.push(String::new());
                }

                units.join("..")
            },
            PatternKind::Range {
                ty,
                from,
                to,
            } => {
                let f = from.render(ty);
                let t = to.render(ty);
                let delim = if to.is_minus_epsilon() {
                    "..~"
                } else {
                    ".."
                };

                format!("{f}{delim}{t}")
            },
            PatternKind::TupleStruct { name, fields } => format!(
                "{}({})",
                name.iter().map(
                    |name| name.id().to_string()
                ).collect::<Vec<_>>().join("."),
                fields.iter().map(
                    |pat| pat.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            p_kind @ (PatternKind::Tuple(patterns)
            | PatternKind::List(patterns)) => format!(
                "{}{}{}",
                if let PatternKind::Tuple(_) = p_kind { "(" } else { "[" },
                patterns.iter().map(
                    |pattern| pattern.to_string()
                ).collect::<Vec<_>>().join(", "),
                if let PatternKind::Tuple(_) = p_kind { ")" } else { "]" },
            ),
            PatternKind::Wildcard => String::from("_"),
            PatternKind::Shorthand => String::from(".."),

            // TODO: no parentheses?
            PatternKind::Or(patterns) => patterns.iter().map(
                |pattern| pattern.to_string()
            ).collect::<Vec<_>>().join("|"),
        };

        write!(fmt, "{result}")
    }
}

impl RenderError for NumberLike {
    fn render_error(&self) -> String {
        match self {
            NumberLike::OpenEnd { .. } => String::new(),  // Do we even need this branch?
            NumberLike::Exact(num)
            | NumberLike::MinusEpsilon(num) => num.to_string(),
        }
    }
}

impl NumberLike {
    pub fn render(&self, ty: &RangeType) -> String {
        match self {
            NumberLike::OpenEnd { .. } => String::new(),  // `..1`
            NumberLike::Exact(n)
            | NumberLike::MinusEpsilon(n) => match ty {
                RangeType::Integer
                | RangeType::Ratio => n.to_string(),
                RangeType::Char => format!("{:?}", n.try_unwrap_small_integer().unwrap() as u8 as char),
            },
        }
    }
}
