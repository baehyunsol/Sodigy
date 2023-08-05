use crate::ast::LocalUIDs;
use crate::err::ParseError;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::Token;

mod err;
mod kind;
mod name_resolve;

#[cfg(test)]
mod tests;

pub use err::PatternErrorKind;
pub use kind::{PatternKind, RangeType};

#[cfg(test)]
pub use tests::is_eq_pat_err;

#[derive(Clone)]
pub struct Pattern {
    kind: PatternKind,
    pub(crate) span: Span,
}

impl Pattern {
    pub fn wildcard(span: Span) -> Self {
        Pattern {
            kind: PatternKind::WildCard,
            span,
        }
    }

    pub fn binding(name: InternedString, span: Span) -> Self {
        Pattern {
            kind: PatternKind::Binding(name),
            span,
        }
    }

    pub fn tuple(patterns: Vec<Pattern>, span: Span) -> Self {
        Pattern {
            kind: PatternKind::Tuple(patterns),
            span,
        }
    }

    pub fn is_tuple(&self) -> bool {
        if let PatternKind::Tuple(_) = self.kind {
            true
        } else {
            false
        }
    }

    pub fn slice(patterns: Vec<Pattern>, span: Span) -> Self {
        Pattern {
            kind: PatternKind::Slice(patterns),
            span,
        }
    }

    pub fn identifier(path: Vec<(InternedString, Span)>) -> Self {
        let span = path[0].1;

        Pattern {
            kind: PatternKind::Identifier(
                Box::new(Path::from_names(path).into_expr())
            ),
            span,
        }
    }

    pub fn enum_tuple(path: Vec<(InternedString, Span)>, patterns: Vec<Pattern>) -> Self {
        let span = path[0].1;

        Pattern {
            kind: PatternKind::EnumTuple(
                Box::new(Path::from_names(path).into_expr()),
                patterns,
            ),
            span,
        }
    }

    pub fn shorthand(span: Span) -> Self {
        Pattern {
            kind: PatternKind::Shorthand,
            span,
        }
    }

    pub fn constant(t: Token) -> Self {
        let span = t.span;
        Pattern {
            kind: PatternKind::Constant(t),
            span,
        }
    }

    pub fn range(t1: Option<Token>, t2: Option<Token>, range_type: RangeType, span: Span) -> Self {
        Pattern {
            kind: PatternKind::Range(t1, t2, range_type),
            span,
        }
    }

    pub fn get_patterns(self) -> Option<Vec<Pattern>> {
        match self.kind {
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns)
            | PatternKind::EnumTuple(_, patterns) => Some(patterns),
            PatternKind::Struct(_, patterns) => Some(patterns.into_iter().map(
                |(_, pattern)| pattern
            ).collect()),
            PatternKind::Shorthand
            | PatternKind::WildCard
            | PatternKind::Identifier(_)
            | PatternKind::Binding(_)
            | PatternKind::Constant(_)
            | PatternKind::Range(_, _, _) => None
        }
    }

    // it doesn't check the refutability of the pattern
    pub fn check_validity(&self) -> Result<(), ParseError> {
        match &self.kind {
            PatternKind::WildCard
            | PatternKind::Shorthand
            | PatternKind::Binding(_)
            | PatternKind::Identifier(_) => Ok(()),
            PatternKind::Range(from, to, range_type) => match (&from, &to) {
                (None, None) => unreachable!("Internal Compiler Error 43CC27E1FF7"),
                (Some(t), None) | (None, Some(t)) => if t.is_character() {
                    Ok(())
                } else {
                    let n = t.unwrap_number();

                    if !n.is_integer() {
                        Err(ParseError::non_integer_in_range(t.clone()))
                    } else {
                        Ok(())
                    }
                },
                (Some(t1), Some(t2)) => {
                    if t1.is_character() && !t2.is_character()
                    || !t1.is_character() && t2.is_character() {
                        return Err(ParseError::unmatched_type_in_range(t1.clone(), t2.clone()));
                    }

                    if t1.is_character() {
                        let s1 = t1.unwrap_character();
                        let mut s2 = t2.unwrap_character();
                        if let RangeType::Inclusive = range_type { s2 += 1; }

                        if s1 >= s2 {
                            return Err(ParseError::invalid_char_range(s1, s2, *range_type, self.span));
                        }
                    } else {
                        let n1 = t1.unwrap_number();
                        let n2 = t2.unwrap_number();

                        if !n1.is_integer() {
                            return Err(ParseError::non_integer_in_range(t1.clone()));
                        }

                        if !n2.is_integer() {
                            return Err(ParseError::non_integer_in_range(t2.clone()));
                        }

                        let mut end = n2.clone();
                        if let RangeType::Inclusive = range_type { end.add_i32_mut(1); }

                        // TODO: warning when n1 + 1 == end

                        if n1.geq_rat(&end) {
                            return Err(ParseError::invalid_int_range(n1.clone(), n2.clone(), *range_type, self.span));
                        }
                    }

                    Ok(())
                },
            },

            PatternKind::Constant(t) => if t.is_character() {
                Ok(())
            } else if t.is_number() {
                let n = t.unwrap_number();

                if !n.is_integer() {
                    return Err(ParseError::non_integer_in_range(t.clone()));
                }

                Ok(())
            } else {
                unreachable!("Internal Compiler Error E4555DA0777")
            },

            PatternKind::Slice(patterns)
            | PatternKind::Tuple(patterns)
            | PatternKind::EnumTuple(_, patterns) => {
                let mut shorthand_spans = vec![];

                for pat in patterns.iter() {
                    pat.check_validity()?;

                    if let PatternKind::Shorthand = &pat.kind {
                        shorthand_spans.push(pat.span);
                    }

                }

                if shorthand_spans.len() > 1 {
                    return Err(ParseError::multiple_shorthand_in_pattern(shorthand_spans));
                }

                Ok(())
            },

            PatternKind::Struct(_, _) => todo!(),
        }
    }

    pub fn get_name_bindings(&self, buffer: &mut Vec<(InternedString, Span)>) {
        match &self.kind {
            PatternKind::WildCard
            | PatternKind::Shorthand
            | PatternKind::Identifier(_)
            | PatternKind::Constant(_)
            | PatternKind::Range(_, _, _) => {},
            PatternKind::Binding(name) => {
                buffer.push((*name, self.span));
            },
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns)
            | PatternKind::EnumTuple(_, patterns) => {
                for pat in patterns.iter() {
                    pat.get_name_bindings(buffer);
                }
            },
            PatternKind::Struct(_, patterns) => {
                for (_, pat) in patterns.iter() {
                    pat.get_name_bindings(buffer);
                }
            },
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        match &self.kind {
            PatternKind::WildCard => "_".to_string(),
            PatternKind::Shorthand => "..".to_string(),
            PatternKind::Constant(t) => t.dump(session),
            PatternKind::Binding(b) => format!("${}", b.to_string(session)),
            PatternKind::Identifier(name) => name.dump(session),
            PatternKind::Tuple(ps)
            | PatternKind::Slice(ps) => {
                let (s, e) = if self.is_tuple() {
                    ("(", ")")
                } else {
                    ("[", "]")
                };

                format!(
                    "{s}{}{e}",
                    ps.iter().map(
                        |pat| pat.dump(session)
                    ).collect::<Vec<String>>().join(",")
                )
            },
            PatternKind::EnumTuple(p, ps) => {
                format!(
                    "{}({})",
                    p.dump(session),
                    ps.iter().map(
                        |pat| pat.dump(session)
                    ).collect::<Vec<String>>().join(",")
                )
            },
            _ => todo!(),
        }
    }

    // read the comments in `sdg_ast::ast::opt::intra_inter_mod`
    // it finds tuple and struct names in patterns, and converts them to `ValueKind::Object(id)`
    pub fn intra_inter_mod(&mut self, session: &LocalParseSession, ctxt: &LocalUIDs) {
        match &mut self.kind {
            PatternKind::WildCard
            | PatternKind::Shorthand
            | PatternKind::Binding(_)
            | PatternKind::Constant(_)
            | PatternKind::Range(_, _, _) => {},
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns) => {
                for pat in patterns.iter_mut() {
                    pat.intra_inter_mod(session, ctxt);
                }
            },
            PatternKind::Identifier(name) => {
                name.intra_inter_mod(session, ctxt);
            },
            PatternKind::EnumTuple(name, patterns) => {
                name.intra_inter_mod(session, ctxt);

                for pat in patterns.iter_mut() {
                    pat.intra_inter_mod(session, ctxt);
                }
            },
            PatternKind::Struct(name, patterns) => {
                name.intra_inter_mod(session, ctxt);

                for (_, pat) in patterns.iter_mut() {
                    pat.intra_inter_mod(session, ctxt);
                }
            },
        }
    }
}
