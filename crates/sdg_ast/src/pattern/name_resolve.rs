use super::{Pattern, PatternKind};
use crate::ast::{NameOrigin, NameScope};
use crate::err::ParseError;
use crate::expr::{Expr, ExprKind, InfixOp};
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::FuncDef;
use crate::value::ValueKind;
use std::collections::{HashSet, HashMap};

impl Pattern {
    // a `Pattern` may include
    //   - enum name, enum variant name, struct name, const
    // a `Pattern` may not include
    //   - local val, func call, 
    // `Some($foo)` -> `Sodigy.Option.Some($foo)`
    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
        used_names: &mut HashSet<(InternedString, NameOrigin)>,
    ) {
        match &mut self.kind {
            PatternKind::WildCard
            | PatternKind::Shorthand
            | PatternKind::Binding(_)
            | PatternKind::Constant(_)
            | PatternKind::Range(_, _, _) => {},
            PatternKind::Identifier(name) => {
                name.resolve_names(name_scope, lambda_defs, session, used_names);

                if let Err(e) = name.is_valid_pattern() {
                    match e {
                        InvalidPatternKind::WrongNameOrigin(name, origin) => {
                            session.add_error(ParseError::pattern_from_arg(name, origin, self.span));
                        },
                        InvalidPatternKind::WrongKind(e) => unreachable!(
                            "Internal Compiler Error ABF025015AD: {}",
                            e.dump(session),
                        ),
                    }
                }
            },
            PatternKind::Tuple(patterns)
            | PatternKind::Slice(patterns) => {
                for pat in patterns.iter_mut() {
                    pat.resolve_names(name_scope, lambda_defs, session, used_names);
                }
            },
            PatternKind::EnumTuple(name, patterns) => {
                name.resolve_names(name_scope, lambda_defs, session, used_names);

                if let Err(e) = name.is_valid_pattern() {
                    match e {
                        InvalidPatternKind::WrongNameOrigin(name, origin) => {
                            session.add_error(ParseError::pattern_from_arg(name, origin, self.span));
                        },
                        InvalidPatternKind::WrongKind(e) => unreachable!(
                            "Internal Compiler Error ABF025015AD: {}",
                            e.dump(session),
                        ),
                    }
                }

                for pat in patterns.iter_mut() {
                    pat.resolve_names(name_scope, lambda_defs, session, used_names);
                }
            }
            PatternKind::Struct(name, patterns) => {
                name.resolve_names(name_scope, lambda_defs, session, used_names);

                if let Err(e) = name.is_valid_pattern() {
                    match e {
                        InvalidPatternKind::WrongNameOrigin(name, origin) => {
                            session.add_error(ParseError::pattern_from_arg(name, origin, self.span));
                        },
                        InvalidPatternKind::WrongKind(e) => unreachable!(
                            "Internal Compiler Error ABF025015AD: {}",
                            e.dump(session),
                        ),
                    }
                }

                for (_, pat) in patterns.iter_mut() {
                    pat.resolve_names(name_scope, lambda_defs, session, used_names);
                }
            },
        }

    }
}


impl Expr {
    pub fn is_valid_pattern(&self) -> Result<(), InvalidPatternKind> {
        match &self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_, NameOrigin::Global)
                | ValueKind::Identifier(_, NameOrigin::Local)
                | ValueKind::Identifier(_, NameOrigin::Prelude)
                | ValueKind::Identifier(_, NameOrigin::NotKnownYet) => Ok(()),
                ValueKind::Identifier(name, origin) => Err(InvalidPatternKind::WrongNameOrigin(*name, origin.clone())),
                ValueKind::Object(_) => Ok(()),
                _ => Err(InvalidPatternKind::WrongKind(self.clone())),
            },
            ExprKind::Infix(InfixOp::Path, id1, _) => id1.is_valid_pattern(),
            _ => Err(InvalidPatternKind::WrongKind(self.clone())),
        }
    }
}

pub enum InvalidPatternKind {
    WrongNameOrigin(InternedString, NameOrigin),
    WrongKind(Expr),
}
