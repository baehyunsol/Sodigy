use super::{Expr, ExprKind, Lambda, Scope};
use crate::IdentWithOrigin;
use crate::func::{Func, FuncKind};
use crate::names::{NameOrigin, NameSpace};
use crate::session::HirSession;
use crate::walker::{EmptyMutWalkerState, MutWalkerState, mut_walker_func};
use sodigy_ast::IdentWithSpan;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::HashSet;

// # here, `fibo` is not a closure
// let fibo10 = {
//     let fibo = \{n, if n < 2 { 1 } else { fibo(n - 1) + fibo(n - 2) }};
//
//     fibo(10)
// };
pub fn try_convert_closures_to_lambdas(f: &mut Func) {
    mut_walker_func(f, &mut EmptyMutWalkerState {}, &Box::new(try_convert_closures_to_lambdas_worker));
}

fn try_convert_closures_to_lambdas_worker(e: &mut Expr, c: &mut EmptyMutWalkerState) {
    match &e.kind {
        ExprKind::Scope(Scope { .. }) => {
            // TODO
        },
        _ => { /* nop */ }
    }
}

pub struct LambdaCollectCtxt<'h> {
    session: &'h mut HirSession,
    pub(crate) collected_lambdas: Vec<Func>,
}

impl<'h> LambdaCollectCtxt<'h> {
    pub fn new(hir_session: &'h mut HirSession) -> Self {
        LambdaCollectCtxt {
            session: hir_session,
            collected_lambdas: vec![],
        }
    }

    pub fn new_name_from_span(&mut self, span: SpanRange) -> InternedString {
        let span_hash = span.hash128();

        // `@` prevents name collisions
        self.session.intern_string(format!("@@LAMBDA_{span_hash:x}").into())
    }
}

impl MutWalkerState for LambdaCollectCtxt<'_> {}

// find lambda functions,
// make them into an actual function,
// and replace lambda expression with the name of the newly created function
pub fn give_names_to_lambdas(f: &mut Func, c: &mut LambdaCollectCtxt) {
    mut_walker_func(f, c, &Box::new(give_names_to_lambdas_worker));
}

fn give_names_to_lambdas_worker(e: &mut Expr, c: &mut LambdaCollectCtxt) {
    match &e.kind {
        ExprKind::Lambda(Lambda {
            args,
            value,
            captured_values,
            uid,
            return_ty: _,
            lowered_from_scoped_let: _,
        }) => {
            if captured_values.is_empty() {
                let uid = uid.into_def();
                let new_name = c.new_name_from_span(e.span);

                let new_func = Func {
                    name: IdentWithSpan::new(
                        new_name,
                        e.span,
                    ),
                    args: Some(args.to_vec()),
                    generics: vec![],
                    return_val: *value.clone(),
                    return_ty: None,
                    attributes: vec![],
                    kind: FuncKind::Lambda,
                    uid,
                };

                e.kind = ExprKind::Identifier(IdentWithOrigin::new(
                    new_name,
                    NameOrigin::Global { origin: Some(uid) },
                ));

                c.collected_lambdas.push(new_func);
            }

            else {
                // TODO: closure
            }
        },
        _ => { /* nop */ },
    }
}

pub struct ValueCaptureCtxt<'c> {
    lambda_uid: Uid,
    captured_values: &'c mut Vec<Expr>,
    used_names: &'c mut HashSet<IdentWithOrigin>,
    name_space: &'c mut NameSpace,
}

impl MutWalkerState for ValueCaptureCtxt<'_> {}

impl<'c> ValueCaptureCtxt<'c> {
    pub fn new(
        lambda_uid: Uid,
        captured_values: &'c mut Vec<Expr>,
        used_names: &'c mut HashSet<IdentWithOrigin>,
        name_space: &'c mut NameSpace,
    ) -> Self {
        ValueCaptureCtxt {
            lambda_uid, captured_values,
            used_names, name_space,
        }
    }
}

pub fn find_and_replace_captured_values(
    e: &mut Expr,
    c: &mut ValueCaptureCtxt,
) {
    match &mut e.kind {
        ExprKind::Identifier(id_ori) => {
            let origin = *id_ori.origin();

            // checks whether this id should be captured or not
            match origin {
                NameOrigin::Prelude   // not captured 
                | NameOrigin::Global { .. }  // not captured
                | NameOrigin::Captured { .. }  // captured, but it'll handle names in Lambda.captured_values
                => {
                    return;
                },
                NameOrigin::FuncArg { .. }
                | NameOrigin::FuncGeneric { .. } => {
                    /* must be captured */
                },
                NameOrigin::Local { origin: local_origin } => {
                    // has to see whether it's captured or not
                    // there are 2 cases: Lambda in a Scope, Scope in a Lambda
                    // first case: that's a captured name and the scope is still in the name_space
                    // second case: that's not a captured name and we can ignore that
                    if !c.name_space.has_this_local_uid(local_origin) {
                        return;
                    }
                },
            }

            let id = id_ori.id();
            let mut name_index = None;

            // linear search is fine because `captured_values` is small enough in most cases
            for (ind, val) in c.captured_values.iter().enumerate() {
                if let ExprKind::Identifier(id_ori_) = &val.kind {
                    let id_ = id_ori_.id();
                    let origin_ = *id_ori_.origin();

                    if (id, origin) == (id_, origin_) {
                        name_index = Some(ind);
                        break;
                    }
                }
            }

            if name_index == None {
                name_index = Some(c.captured_values.len());
                c.captured_values.push(Expr {
                    kind: ExprKind::Identifier(IdentWithOrigin::new(id, origin)),
                    span: e.span,
                });
            }

            let name_index = name_index.unwrap();
            id_ori.set_origin(NameOrigin::Captured {
                lambda: c.lambda_uid,
                index: name_index,
            });
            c.used_names.insert(id_ori.clone());
        },
        _ => {},
    }
}
