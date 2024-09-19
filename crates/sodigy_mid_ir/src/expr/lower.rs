use super::{Expr, ExprKind, MirFunc};
use crate::session::{LocalValueSearchKey, MirSession};
use crate::ty::lower_ty;
use sodigy_high_ir::{self as hir, NameOrigin};

pub fn lower_expr(
    expr: &hir::Expr,
    ty: Option<&hir::Type>,
    tail_call: bool,
    session: &mut MirSession,
) -> Result<Expr, ()> {
    if let hir::ExprKind::Scope(hir::Scope {
        original_patterns,  // TODO: lower this and save somewhere
        value,
        ..
    }) = &expr.kind {
        // scoped-lets are already collected

        lower_expr(
            value.as_ref(),
            ty,
            tail_call,
            session,
        )
    }

    else {
        let kind = lower_expr_kind(
            &expr.kind,
            tail_call,
            session,
        );
        let ty = ty.map(
            |ty| lower_ty()
        );

        Ok(Expr {
            kind: kind?,
            span: expr.span,
            ty: if let Some(ty) = ty { Some(ty?) } else { None },
        })
    }
}

pub fn lower_expr_kind(
    kind: &hir::ExprKind,
    tail_call: bool,
    session: &mut MirSession,
) -> Result<ExprKind, ()> {
    let k = match kind {
        hir::ExprKind::Identifier(id_with_origin) => match id_with_origin.origin() {
            NameOrigin::Prelude(uid)
            | NameOrigin::LangItem(uid) => ExprKind::Object(*uid),

            // hir's name resolution must have removed all the `None`s
            NameOrigin::Global { origin } => ExprKind::Object(origin.unwrap()),

            NameOrigin::Local {
                origin, index: _, binding_type: _,
            } => ExprKind::LocalValue {
                origin: session.curr_func_uid(),
                key: session.get_local_value_index(LocalValueSearchKey::LocalValue(*origin, id_with_origin.id())),
            },
            NameOrigin::FuncArg { .. } => ExprKind::LocalValue {
                origin: session.curr_func_uid(),
                key: session.get_local_value_index(LocalValueSearchKey::FuncArg(id_with_origin.id())),
            },
            NameOrigin::FuncGeneric { .. } => ExprKind::LocalValue {
                origin: session.curr_func_uid(),
                key: session.get_local_value_index(LocalValueSearchKey::FuncGeneric(id_with_origin.id())),
            },
            NameOrigin::Captured { .. } => todo!(),
        },
        hir::ExprKind::Integer(n) => ExprKind::Integer(*n),
        hir::ExprKind::Call {
            func, args,
        } => {
            let func = lower_expr(
                func.as_ref(),
                None,
                false,
                session,
            );
            let mut mir_args = Vec::with_capacity(args.len());
            let mut has_error = false;

            for arg in args.iter() {
                if let Ok(mir_arg) = lower_expr(
                    arg,
                    None,
                    false,
                    session,
                ) {
                    mir_args.push(mir_arg);
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            let func = func?;
            let func = if let ExprKind::Object(uid) = &func.kind {
                MirFunc::Static(*uid)
            } else {
                MirFunc::Dynamic(Box::new(func))
            };

            ExprKind::Call {
                func,
                args: mir_args,
                tail_call,
            }
        },
        hir::ExprKind::List(elements) => {
            let mut mir_elements = Vec::with_capacity(elements.len());
            let mut has_error = false;

            for element in elements.iter() {
                if let Ok(mir_element) = lower_expr(
                    element,
                    None,
                    false,
                    session,
                ) {
                    mir_elements.push(mir_element);
                } else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            ExprKind::Call {
                func: MirFunc::Static(todo!() /* Prelude::init_list */ ),
                args: mir_elements,
                tail_call,
            }
        },
        // see `lower_expr`
        hir::ExprKind::Scope(_) => unreachable!(),
        _ => panic!("TODO: {kind}"),
    };

    Ok(k)
}
