use super::{Expr, ExprKind, MirFunc};
use crate::error::MirError;
use crate::session::{LocalValueSearchKey, MirSession};
use crate::ty::lower_ty;
use sodigy_high_ir::{self as hir, NameOrigin};
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

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
            expr.span,
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
    span: SpanRange,  // of hir::Expr
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
        ekind @ (hir::ExprKind::List(elements)
        | hir::ExprKind::Tuple(elements)
        | hir::ExprKind::Format(elements)) => {
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

            let func_uid = match ekind {
                hir::ExprKind::List(_) => Uid::dummy(1),  // TODO
                hir::ExprKind::Tuple(_) => Uid::dummy(2),  // TODO
                hir::ExprKind::Format(_) => Uid::dummy(3),  // TODO
                _ => unreachable!(),
            };

            ExprKind::Call {
                func: MirFunc::Static(func_uid),
                args: mir_elements,
                tail_call,
            }
        },
        // see `lower_expr`
        hir::ExprKind::Scope(_) => unreachable!(),
        hir::ExprKind::StructInit(hir::StructInit {
            struct_, fields,
        }) => {
            let mut has_error = false;
            let mir_struct = lower_expr(
                struct_.as_ref(),
                None,
                false,
                session,
            );

            let mut mir_fields = Vec::with_capacity(fields.len());

            for field in fields.iter() {
                if let Ok(f) = lower_expr(
                    &field.value,
                    None,
                    false,
                    session,
                ) {
                    mir_fields.push((field.name, f));
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            let mir_struct = mir_struct?;

            match &mir_struct.kind {
                ExprKind::Object(uid) => match session.get_struct_info(*uid) {
                    Some(hir::StructInfo {
                        struct_name,
                        field_names,
                        constructor_uid,
                        ..
                    }) => {
                        let struct_name = *struct_name;
                        let constructor_uid = *constructor_uid;
                        let field_names = field_names.clone();
                        let mut args = Vec::with_capacity(field_names.len());
                        let mut missing_fields = vec![];
                        let mut unknown_fields = vec![];

                        for field_name in field_names.iter() {
                            if let Some(field) = mir_fields.iter().filter(
                                |(name, _)| name.id() == *field_name
                            ).next() {
                                args.push(field.1.clone());
                            }

                            else {
                                missing_fields.push(*field_name);
                            }
                        }

                        for (name, _) in mir_fields.iter() {
                            if !field_names.contains(&name.id()) {
                                unknown_fields.push(*name);
                            }
                        }

                        if !missing_fields.is_empty() {
                            session.push_error(MirError::missing_fields_in_struct_constructor(span, missing_fields, struct_name.id()));
                            has_error = true;
                        }

                        if !unknown_fields.is_empty() {
                            session.push_error(MirError::unknown_fields_in_struct_constructor(unknown_fields, field_names.clone(), struct_name.id()));
                            has_error = true;
                        }

                        if has_error {
                            return Err(());
                        }

                        ExprKind::Call {
                            func: MirFunc::Static(constructor_uid),
                            args,
                            tail_call,
                        }
                    },
                    None => {
                        session.push_error(MirError::not_a_struct(struct_));
                        return Err(());
                    },
                },

                // TODO: are you sure that it's always an error?
                _ => {
                    session.push_error(MirError::not_a_struct(struct_));
                    return Err(());
                },
            }
        },
        _ => panic!("TODO: {kind}"),
    };

    Ok(k)
}
