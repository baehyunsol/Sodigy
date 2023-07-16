use super::{Expr, ExprKind};
use crate::ast::{ASTError, NameScope, NameScopeKind};
use crate::expr::InfixOp;
use crate::session::InternedString;
use crate::stmt::{ArgDef, FuncDef};
use crate::value::ValueKind;
use std::collections::HashMap;

impl Expr {
    pub fn resolve_names(&mut self, name_scope: &mut NameScope, lambda_defs: &mut HashMap<InternedString, FuncDef>) -> Result<(), ASTError> {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {
                    Ok(())
                },
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for elem in elements.iter_mut() {
                        elem.resolve_names(name_scope, lambda_defs)?;
                    }

                    Ok(())
                },
                ValueKind::Identifier(id, _) => match name_scope.search_name(*id) {
                    Ok((None, origin)) => {
                        self.kind.set_origin(origin);

                        Ok(())
                    },
                    Ok((Some(alias), _)) => {
                        // its origin is handled by `.to_path`
                        self.kind = alias.to_path();

                        Ok(())
                    },
                    Err(()) => Err(ASTError::no_def(*id, self.span, name_scope.clone())),
                },
                ValueKind::Lambda(args, expr) => {

                    // TODO: `name_scope.push_names` after `ty.resolve_names`?
                    // -> dependent types?
                    name_scope.push_names(args, NameScopeKind::LambdaArg);

                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_names(name_scope, lambda_defs)?;
                        }
                    }

                    expr.resolve_names(name_scope, lambda_defs)?;
                    name_scope.pop_names();

                    // TODO: initiate new `FuncDef` with this lambda_def
                    // push the newly generate `FuncDef` to `lambda_defs`
                    // replace this `Expr` with the newly generated functor
                    // let lambda_def = FuncDef::from_args_and_val(args.clone(), expr.clone());

                    // if lambda_def.is_closure() {
                    //     todo!();
                    // }

                    // TODO: 새 람다 이름 앞에다가 이상한 특수문자 붙일 거여서 프로그래머가 쓴 이름하고 겹칠 일은 없음. 근데 지들끼리 hash가 겹칠 수는 있음!
                    // if let Some(_) = lambda_defs.insert(lambda_def.name, lambda_def) {
                    //     todo!();
                    // }

                    // *self = lambda_def

                    Ok(())
                },
                ValueKind::Block { defs, value, id } => {
                    name_scope.push_names(defs, NameScopeKind::Block(*id));

                    for (_, expr) in defs.iter_mut() {
                        expr.resolve_names(name_scope, lambda_defs)?;
                    }

                    value.resolve_names(name_scope, lambda_defs)?;
                    name_scope.pop_names();

                    Ok(())
                },
            },
            ExprKind::Prefix(_, operand) | ExprKind::Postfix(_, operand) => operand.resolve_names(name_scope, lambda_defs),
            ExprKind::Branch(cond, b1, b2) => {
                cond.resolve_names(name_scope, lambda_defs)?;
                b1.resolve_names(name_scope, lambda_defs)?;
                b2.resolve_names(name_scope, lambda_defs)?;

                Ok(())
            }
            ExprKind::Call(f, args) => {
                f.resolve_names(name_scope, lambda_defs)?;

                for arg in args.iter_mut() {
                    arg.resolve_names(name_scope, lambda_defs)?;
                }

                Ok(())
            }
            ExprKind::Infix(op, o1, o2) => match op {

                // `a.b.c` -> `a` has to be resolved, but the others shall not
                InfixOp::Path => {
                    o1.resolve_names(name_scope, lambda_defs)?;

                    Ok(())
                },
                _ => {
                    o1.resolve_names(name_scope, lambda_defs)?;
                    o2.resolve_names(name_scope, lambda_defs)?;

                    Ok(())
                }
            },
        }
    }
}