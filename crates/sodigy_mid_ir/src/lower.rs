use crate::def::Def;
use crate::expr::{Expr, ExprKind};
use crate::prelude::{PreludeData, uids};
use crate::ty::Type;
use crate::ty_class::TypeClassQuery;
use sodigy_high_ir as hir;
use sodigy_intern::InternedString;
use sodigy_uid::Uid;
use std::collections::HashMap;

pub fn lower_hir_expr(
    e: &hir::Expr,
    preludes: &HashMap<InternedString, PreludeData>,
    global_defs: &HashMap<Uid, Def>,
    type_annotation: &Option<Type>,  // it's lowered by the caller
    type_classes: &TypeClassQuery,
) -> Result<Expr, ()> {
    let res = match &e.kind {
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
                hir::NameOrigin::Global { origin: Some(uid) } => Expr {
                    kind: ExprKind::Global(uid),
                    ty: global_defs.get(&uid).unwrap().ty.clone(),
                    span: e.span,
                },
                hir::NameOrigin::Global { origin: None } => {
                    // search this name in some table,
                    // then figure out def and uid of thie name
                    todo!()
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
        hir::ExprKind::String { s, is_binary } => if *is_binary {
            Expr::new_bytes(s).set_span(e.span).to_owned()
        } else {
            Expr::new_string(s).set_span(e.span).to_owned()
        },
        hir::ExprKind::Call {
            func, args,
        } => {
            let func = lower_hir_expr(
                func.as_ref(),
                preludes,
                global_defs,
                &None,  // you cannot annotate type here
                type_classes,
            );
            let mut mir_args = Vec::with_capacity(args.len());

            for arg in args.iter() {
                if let Ok(mir_arg) = lower_hir_expr(
                    arg,
                    preludes,
                    global_defs,
                    &None,  // you cannot annotate type here
                    type_classes,
                ) {
                    mir_args.push(mir_arg);
                } else {
                    // TODO: has error
                }
            }

            // TODO: if func is Ok(Expr { kind: ExprKind::Global(id) }), instantiate ExprKind::Call
            // otherwise, it's ExprKind::DynCall
            todo!()
        },
        hir::ExprKind::List(elements) => {
            let mut result = Vec::with_capacity(elements.len());
            let elem_ty_anno = match type_annotation {
                Some(ty) if let Some(elem_ty) = ty.is_list_of() => {
                    Some(elem_ty.clone())
                },

                // it's a type error, but we don't care about that now
                // it'll be caught later
                _ => None,
            };

            for element in elements.iter() {
                if let Ok(e) = lower_hir_expr(
                    element,
                    preludes,
                    global_defs,
                    &elem_ty_anno,
                    type_classes,
                ) {
                    result.push(e);
                }

                else {
                    // TODO: has error
                }
            }

            let ty = if result.is_empty() {
                // TODO: set type of an empty list
                // if there's a type annotation, use that
                // otherwise... then what?
                todo!()
            }

            else {
                // TODO: check type
                todo!();

                Type::Param(
                    uids::LIST_DEF,
                    vec![result[0].ty.clone()],
                )
            };

            Expr {
                kind: ExprKind::Call {
                    f: uids::LIST_INIT,
                    args: result,
                },
                ty,
                span: e.span,
            }
        },
        hir::ExprKind::InfixOp(op, rhs, lhs) => {
            let rhs = lower_hir_expr(
                rhs.as_ref(),
                preludes,
                global_defs,
                &None,  // you cannot annotate type here
                type_classes,
            );
            let lhs = lower_hir_expr(
                lhs.as_ref(),
                preludes,
                global_defs,
                &None,  // you cannot annotate type here
                type_classes,
            );

            let (rhs, lhs) = match (rhs, lhs) {
                (Ok(rhs), Ok(lhs)) => (rhs, lhs),
                _ => {
                    // TODO: Error
                    todo!()
                },
            };

            let f = if let Some(f) = type_classes.query_2_args((*op).into(), &rhs.ty, &lhs.ty) {
                f
            } else {
                // TODO: Error, this trait is not implemented for these types
                todo!()
            };

            Expr {
                kind: ExprKind::Call {
                    f: f.uid,
                    args: vec![rhs, lhs],
                },
                ty: f.ty.clone(),
                span: e.span,
            }
        },
        _ => todo!(),
    };

    if let Some(ty) = type_annotation {
        // TODO: see if type_annotation and res.ty matches
    }

    Ok(res)
}
