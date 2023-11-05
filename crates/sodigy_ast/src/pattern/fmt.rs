use super::PatternKind;

impl PatternKind {
    /// `fmt::Display` for error messages
    pub fn render_error(&self) -> String {
        match self {
            PatternKind::Identifier(id) => format!("{id}"),
            PatternKind::Binding(id) => format!("${id}"),
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
            | PatternKind::Slice(patterns)) => {
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
                    |p| format!("{}", p.id())
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
