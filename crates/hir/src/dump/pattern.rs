use crate::{Pattern, PatternKind, Session};
use sodigy_endec::IndentedLines;

pub fn dump_pattern(pattern: &Pattern, lines: &mut IndentedLines, session: &Session) {
    if let Some(name) = pattern.name {
        lines.push(&name.unintern_or_default(&session.intermediate_dir));
        lines.push(" @ ");
    }

    dump_pattern_kind(&pattern.kind, lines, session);
}

pub fn dump_pattern_kind(pattern_kind: &PatternKind, lines: &mut IndentedLines, session: &Session) {
    match pattern_kind {
        PatternKind::Path(p) => {
            lines.push(&p.unintern_or_default(&session.intermediate_dir));
        },
        PatternKind::NameBinding { id, .. } => {
            lines.push("$");
            lines.push(&id.unintern_or_default(&session.intermediate_dir));
        },
        PatternKind::Number { n, .. } => {
            lines.push(&n.dump());
        },
        PatternKind::String { binary, s, .. } => {
            let s = format!(
                "{}{:?}",
                if *binary { "b" } else { "" },
                s.unintern_or_default(&session.intermediate_dir),
            );
            lines.push(&s);
        },
        PatternKind::Regex { .. } => {
            lines.push(&format!("/* TODO: dump regex pattern {pattern_kind:?} */"));
        },
        PatternKind::Char { ch, .. } => {
            lines.push(&format!("{:?}", char::from_u32(*ch).unwrap()));
        },
        PatternKind::Byte { b, .. } => {
            lines.push(&format!("#{b}"));
        },
        PatternKind::Struct { r#struct, fields, rest, .. } => {
            lines.push(&r#struct.unintern_or_default(&session.intermediate_dir));
            lines.push("{");
            lines.inc_indent();
            lines.break_line();

            for (i, field) in fields.iter().enumerate() {
                lines.push(&field.name.unintern_or_default(&session.intermediate_dir));

                if !field.is_shorthand {
                    lines.push(": ");
                    dump_pattern(&field.pattern, lines, session);
                }

                lines.push(",");

                if i != fields.len() - 1 {
                    lines.break_line();
                }
            }
        },
        PatternKind::TupleStruct { elements, rest, .. } |
        PatternKind::Tuple { elements, rest, .. } |
        PatternKind::List { elements, rest, .. } => {
            if let PatternKind::TupleStruct { r#struct, .. } = pattern_kind {
                lines.push(&r#struct.unintern_or_default(&session.intermediate_dir));
            }

            let is_tuple = matches!(pattern_kind, PatternKind::TupleStruct { .. } | PatternKind::Tuple { .. });
            lines.push(if is_tuple { "(" } else { "[" });
            let element_per_line = lookahead_elements(&elements, session) > 20;

            if elements.len() > 1 {
                if element_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, element) in elements.iter().enumerate() {
                    if let Some(rest) = rest && rest.index == i {
                        lines.push("..,");

                        if element_per_line {
                            lines.break_line();
                        }

                        else {
                            lines.push(" ");
                        }
                    }

                    dump_pattern(&element, lines, session);
                    lines.push(",");

                    if i != elements.len() - 1 {
                        if element_per_line {
                            lines.break_line();
                        }

                        else {
                            lines.push(" ");
                        }
                    }
                }

                if let Some(rest) = rest && rest.index == elements.len() {
                    if element_per_line {
                        lines.break_line();
                    }

                    else {
                        lines.push(" ");
                    }

                    lines.push("..,");
                }

                if element_per_line {
                    lines.dec_indent();
                    lines.break_line();
                }
            }

            else {
                if let Some(rest) = rest && rest.index == 0 {
                    lines.push("..,");
                }

                for element in elements.iter() {
                    dump_pattern(&element, lines, session);
                    lines.push(",");
                }

                if let Some(rest) = rest && rest.index == 1 {
                    lines.push("..,");
                }
            }

            lines.push(if is_tuple { ")" } else { "]" });
        },
        PatternKind::Range { lhs, rhs, is_inclusive, .. } => {
            if let Some(lhs) = lhs {
                dump_pattern(lhs, lines, session);
            }

            lines.push(if *is_inclusive { "..=" } else { ".." });

            if let Some(rhs) = rhs {
                dump_pattern(rhs, lines, session);
            }
        },
        PatternKind::Or { lhs, rhs, .. } => {
            dump_pattern(lhs, lines, session);
            lines.push(" | ");
            dump_pattern(lhs, lines, session);
        },
        PatternKind::Wildcard(_) => {
            lines.push("_");
        },
    }
}

fn lookahead_elements(elements: &[Pattern], session: &Session) -> usize {
    let mut count = 0;

    for element in elements.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_pattern(element, &mut indented_lines, session);
        count += indented_lines.dump().len();
    }

    count
}
