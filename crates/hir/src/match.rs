use crate::{Expr, Pattern, Session};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{NameKind, Namespace, UseCount};
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub value: Box<Expr>,
    pub branches: Vec<MatchBranch>,
}

#[derive(Clone, Debug)]
pub struct MatchBranch {
    pub pattern: Pattern,
    pub cond: Option<Expr>,
    pub value: Expr,
}

impl Match {
    pub fn from_ast(ast_match: &ast::Match, session: &mut Session) -> Result<Match, ()> {
        let mut has_error = false;
        let mut branches = vec![];

        let value = match Expr::from_ast(ast_match.value.as_ref(), session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        for ast_branch in ast_match.branches.iter() {
            let pattern = match Pattern::from_ast(&ast_branch.pattern, session) {
                Ok(pattern) => Some(pattern),
                Err(()) => {
                    has_error = true;
                    None
                },
            };
            let names = ast_branch.pattern.bound_names().iter().map(
                |(id, span)| (*id, (*span, NameKind::PatternNameBind, UseCount::new()))
            ).collect();

            session.name_stack.push(Namespace::Pattern { names });

            let cond = match ast_branch.cond.as_ref().map(|cond| Expr::from_ast(cond, session)) {
                Some(Ok(cond)) => Some(cond),
                Some(Err(())) => {
                    has_error = true;
                    None
                },
                None => None,
            };
            let value = match Expr::from_ast(&ast_branch.value, session) {
                Ok(value) => Some(value),
                Err(()) => {
                    has_error = true;
                    None
                },
            };

            let Some(Namespace::Pattern { names }) = session.name_stack.pop() else { unreachable!() };

            for (name, (span, kind, count)) in names.iter() {
                if count.is_zero() {
                    session.warnings.push(Warning {
                        kind: WarningKind::UnusedName {
                            name: *name,
                            kind: *kind,
                        },
                        span: *span,
                        ..Warning::default()
                    });
                }
            }

            if !has_error {
                branches.push(MatchBranch {
                    pattern: pattern.unwrap(),
                    cond,
                    value: value.unwrap(),
                });
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Match {
                keyword_span: ast_match.keyword_span,
                value: Box::new(value.unwrap()),
                branches,
            })
        }
    }
}
