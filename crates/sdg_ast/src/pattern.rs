use crate::ast::NameScope;
use crate::err::ParseError;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::Token;

mod err;
mod kind;

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

    pub fn slice(patterns: Vec<Pattern>, span: Span) -> Self {
        Pattern {
            kind: PatternKind::Slice(patterns),
            span,
        }
    }

    pub fn path(path: Vec<(InternedString, Span)>) -> Self {
        let span = path[0].1;

        Pattern {
            kind: PatternKind::Path(Path::from_names(path)),
            span,
        }
    }

    pub fn enum_tuple(path: Vec<(InternedString, Span)>, patterns: Vec<Pattern>) -> Self {
        let span = path[0].1;

        Pattern {
            kind: PatternKind::EnumTuple(Path::from_names(path), patterns),
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
            | PatternKind::Path(_)
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
            | PatternKind::Path(_) => Ok(()),
            PatternKind::Range(from, to, range_type) => match (&from, &to) {
                (None, None) => unreachable!("Internal Compiler Error 43CC27E1FF7"),
                (Some(t), None) | (None, Some(t)) => if t.is_string() {
                    let s = t.unwrap_string();

                    if s.len() != 1 {
                        Err(ParseError::only_char_in_range(t.clone()))
                    } else {
                        Ok(())
                    }
                } else {
                    let n = t.unwrap_number();

                    if !n.is_integer() {
                        Err(ParseError::non_integer_in_range(t.clone()))
                    } else {
                        Ok(())
                    }
                },
                (Some(t1), Some(t2)) => {
                    if t1.is_string() && !t2.is_string()
                    || !t1.is_string() && t2.is_string() {
                        return Err(ParseError::unmatched_type_in_range(t1.clone(), t2.clone()));
                    }

                    if t1.is_string() {
                        let s1 = t1.unwrap_string();
                        let s2 = t2.unwrap_string();

                        if s1.len() != 1 {
                            return Err(ParseError::only_char_in_range(t1.clone()));
                        }

                        if s2.len() != 1 {
                            return Err(ParseError::only_char_in_range(t2.clone()));
                        }

                        let mut end = s2[0];
                        if let RangeType::Inclusive = range_type { end += 1; }

                        if s1[0] >= end {
                            return Err(ParseError::invalid_char_range(s1[0], s2[0], *range_type, self.span));
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

            PatternKind::Constant(t) => if t.is_string() {
                let s = t.unwrap_string();

                if s.len() != 1 {
                    return Err(ParseError::only_char_in_range(t.clone()));
                }

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
            | PatternKind::Path(_)
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
        todo!()
    }

    // a `Pattern` may include
    //   - enum name, enum variant name, struct name, const
    // a `Pattern` may not include
    //   - local val, func call, 
    // `Some($foo)` -> `Sodigy.Option.Some($foo)`
    pub fn resolve_names(&mut self, scope: &NameScope, session: &mut LocalParseSession) {
        match &mut self.kind {
            PatternKind::WildCard
            | PatternKind::Shorthand
            | PatternKind::Binding(_)
            | PatternKind::Constant(_)
            | PatternKind::Range(_, _, _) => {},
            PatternKind::Path(p) => {
                p.resolve_names(scope, session);
            },
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns) => {
                for pat in patterns.iter_mut() {
                    pat.resolve_names(scope, session);
                }
            },
            PatternKind::EnumTuple(path, patterns) => {
                path.resolve_names(scope, session);

                for pat in patterns.iter_mut() {
                    pat.resolve_names(scope, session);
                }
            }
            PatternKind::Struct(path, patterns) => {
                path.resolve_names(scope, session);

                for (_, pat) in patterns.iter_mut() {
                    pat.resolve_names(scope, session);
                }
            },
        }
    }
}
