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
        PatternKind::Ident { id, .. } => {
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
        PatternKind::Path(ids) => {
            lines.push(&ids.iter().map(
                |(id, _)| id.unintern_or_default(&session.intermediate_dir)
            ).collect::<Vec<_>>().join("."));
        },
        PatternKind::Tuple { elements, rest, .. } | PatternKind::List { elements, rest, .. } => {
            let is_tuple = matches!(pattern_kind, PatternKind::Tuple { .. });
            lines.push(if is_tuple { "(" } else { "[" });

            if elements.len() > 1 {
                lines.inc_indent();
                lines.break_line();

                for (i, element) in elements.iter().enumerate() {
                    if let Some(rest) = rest && rest.index == i {
                        lines.push("..,");
                        lines.break_line();
                    }

                    dump_pattern(&element, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
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
        _ => todo!(),
    }
}
