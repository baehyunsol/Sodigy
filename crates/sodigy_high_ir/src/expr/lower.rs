use super::{Expr, ExprKind};
use crate::err::HirError;
use crate::names::{NameOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan, ValueKind};
use sodigy_intern::InternedString;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_expr(
    e: &ast::Expr,
    session: &mut HirSession,
    used_names: &mut HashSet<(InternedString, NameOrigin)>,

    // `use z as x.y.z;` -> {'z': ['x', 'y', 'z']}
    use_cases: &HashMap<InternedString, Vec<InternedString>>,

    name_space: &mut NameSpace,
) -> Result<Expr, ()> {
    let res = match &e.kind {
        ast::ExprKind::Value(v) => match &v {
            ValueKind::Identifier(id) => {
                if let Some(u) = use_cases.get(id) {
                    // unfold u
                    todo!()
                }

                else if let Some(origin) = name_space.find_origin(*id) {
                    used_names.insert((*id, origin));

                    Expr {
                        kind: ExprKind::Identifier(*id, origin),
                        span: e.span,
                    }
                }

                else {
                    session.push_error(HirError::undefined_name(
                        IdentWithSpan::new(*id, e.span),
                        vec![],  // TODO: suggestions
                    ));
                    return Err(());
                }
            },
            ValueKind::Number(n) => todo!(),
            ValueKind::String { s, is_binary } => todo!(),
            ValueKind::Char(c) => todo!(),
            ValueKind::List(elems) => todo!(),
            ValueKind::Tuple(elems) => todo!(),
            ValueKind::Format(elems) => todo!(),
            ValueKind::Lambda {
                args, value,
            } => {
                // Push names defined in this lambda, then recurs
                todo!();

                // check unused-names
                todo!();
            },
            ValueKind::Scope(scope) => {
                // Push names defined in this scope, then recurs
                todo!();

                // check unused-names
                todo!();
            },
        },
        ast::ExprKind::PrefixOp(op, expr) => Expr {
            kind: ExprKind::PrefixOp(
                *op,
                Box::new(lower_ast_expr(
                    expr,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                )?),
            ),
            span: e.span,
        },
        ast::ExprKind::PostfixOp(op, expr) => Expr {
            kind: ExprKind::PostfixOp(
                *op,
                Box::new(lower_ast_expr(
                    expr,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                )?),
            ),
            span: e.span,
        },
        ast::ExprKind::InfixOp(op, lhs, rhs) => {
            // we should unwrap these after both lowerings are complete
            // it helps us find more errors in case there are ones
            let lhs = lower_ast_expr(
                lhs,
                session,
                used_names,
                use_cases,
                name_space,
            );
            let rhs = lower_ast_expr(
                rhs,
                session,
                used_names,
                use_cases,
                name_space,
            );

            Expr {
                kind: ExprKind::InfixOp(
                    *op,
                    Box::new(lhs?),
                    Box::new(rhs?),
                ),
                span: e.span,
            }
        },
        ast::ExprKind::Path { pre, post } => todo!(),
        ast::ExprKind::Call { functor, args } => todo!(),
        ast::ExprKind::StructInit { struct_, init } => todo!(),
        ast::ExprKind::Branch(arms) => {
            // Push names defined in the arms (if there's `if let`), then recurs
            todo!();

            // check unused-names
            todo!();
        },
        ast::ExprKind::Match { value, arms } => {
            // Push names defined in the arms, then recurs
            todo!();

            // check unused-names
            todo!();
        },
    };

    Ok(res)
}
