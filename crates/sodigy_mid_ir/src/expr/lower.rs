use super::{Expr, ExprKind};
use crate::{
    prelude::{PreludeData, uids},
    session::MirSession,
    ty::Type,
    ty_class::TypeClass,
};
use sodigy_high_ir as hir;
use sodigy_intern::InternedString;
use std::collections::HashMap;

// it lowers hir to mir, but doesn't do anything regarding types unless the type is obvious
// all the type errors are caught later
pub fn lower_hir_expr_without_types(
    e: &hir::Expr,
    session: &mut MirSession,
    preludes: &HashMap<InternedString, PreludeData>,
) -> Expr {
    match &e.kind {
        hir::ExprKind::Identifier(origin) => {
            let id = origin.id();
            let origin = *origin.origin();

            match origin {
                hir::NameOrigin::Prelude => {
                    let prelude_data = preludes.get(&id).unwrap();

                    Expr {
                        kind: ExprKind::Global(prelude_data.uid),
                        ty: prelude_data.ty.clone(),
                        span: e.span,
                    }
                },
                hir::NameOrigin::Global { origin } => match origin {
                    Some(uid) => Expr {
                        kind: ExprKind::Global(uid),
                        ty: Type::HasToBeInfered,
                        span: e.span,
                    },
                    None => todo!(),
                },
                _ => todo!(),
            }
        },
        hir::ExprKind::Integer(n) => Expr::new_int(*n).set_span(e.span).to_owned(),

        // `1.75` is lowered to `Ratio.init(4, 7)`
        hir::ExprKind::Ratio(n) => {
            let (denom, numer) = n.get_denom_and_numer();

            Expr {
                kind: ExprKind::Call {
                    f: uids::RATIO_INIT,
                    args: vec![
                        Expr::new_int(denom),
                        Expr::new_int(numer),
                    ],
                },
                ty: Type::Solid(uids::RATIO_DEF),
                span: e.span,
            }
        },
        hir::ExprKind::Char(c) => Expr::new_char(*c).set_span(e.span).to_owned(),
        hir::ExprKind::String { content, is_binary } => if *is_binary {
            Expr::new_bytes(content).set_span(e.span).to_owned()
        } else {
            Expr::new_string(content).set_span(e.span).to_owned()
        },
        hir::ExprKind::Call {
            func, args,
        } => {
            let func = lower_hir_expr_without_types(
                func.as_ref(),
                session,
                preludes,
            );
            let mir_args = args.iter().map(
                |arg| lower_hir_expr_without_types(
                    arg,
                    session,
                    preludes,
                )
            ).collect::<Vec<Expr>>();

            match &func.kind {
                ExprKind::Global(uid) => Expr {
                    kind: ExprKind::Call {
                        f: *uid,
                        args: mir_args,
                    },
                    ty: Type::HasToBeInfered,
                    span: e.span,
                },
                _ => Expr {
                    kind: ExprKind::DynCall {
                        f: Box::new(func),
                        args: mir_args,
                    },
                    ty: Type::HasToBeInfered,
                    span: e.span,
                },
            }
        },

        // `[1, 2, 3]` is lowered to `list_init(1, 2, 3)`
        hir::ExprKind::List(elements) => Expr {
            kind: ExprKind::Call {
                f: uids::LIST_INIT,
                args: elements.iter().map(
                    |element| lower_hir_expr_without_types(
                        element,
                        session,
                        preludes,
                    )
                ).collect(),
            },
            ty: Type::Param(
                uids::LIST_DEF,
                vec![Type::HasToBeInfered],
            ),
            span: e.span,
        },

        // `"{a} + {b} = {a + b}"` is lowered to `concat_all(a.to_string(), " + ", b.to_string(), " = ", (a + b).to_string())`
        hir::ExprKind::Format(elements) => {
            let mut result = Vec::with_capacity(elements.len());

            for element in elements.iter() {
                let e = lower_hir_expr_without_types(
                    element,
                    session,
                    preludes,
                );

                if e.is_obviously_string() {
                    result.push(e);
                }

                else {
                    let span = e.span;
                    result.push(
                        Expr {
                            kind: ExprKind::Call {
                                f: TypeClass::ToString.generic_uid(),
                                args: vec![e],
                            },
                            ty: Type::Solid(uids::STRING_DEF),
                            span,
                        }
                    );
                }
            }

            Expr {
                kind: ExprKind::Call {
                    f: todo!(),  // ConcatAll(List(Any))
                    args: result,
                },
                ty: Type::Solid(uids::STRING_DEF),
                span: e.span,
            }
        },
        hir::ExprKind::PrefixOp(op, val) => Expr {
            kind: ExprKind::Call {
                f: TypeClass::from(*op).generic_uid(),
                args: vec![lower_hir_expr_without_types(
                    val,
                    session,
                    preludes,
                )],
            },
            ty: Type::HasToBeInfered,
            span: e.span,
        },
        hir::ExprKind::PostfixOp(op, val) => Expr {
            kind: ExprKind::Call {
                f: TypeClass::from(*op).generic_uid(),
                args: vec![lower_hir_expr_without_types(
                    val,
                    session,
                    preludes,
                )],
            },
            ty: Type::HasToBeInfered,
            span: e.span,
        },
        hir::ExprKind::InfixOp(op, lhs, rhs) => Expr {
            kind: ExprKind::Call {
                f: TypeClass::from(*op).generic_uid(),
                args: vec![
                    lower_hir_expr_without_types(
                        lhs,
                        session,
                        preludes,
                    ),
                    lower_hir_expr_without_types(
                        rhs,
                        session,
                        preludes,
                    ),
                ],
            },
            ty: Type::HasToBeInfered,
            span: e.span,
        },
        _ => todo!(),
    }
}
