use super::{Expr, ExprKind};
use crate::ast::{ASTError, NameScope, NameScopeId, NameScopeKind};
use crate::expr::InfixOp;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{ArgDef, FuncDef};
use crate::value::ValueKind;
use std::collections::HashMap;

impl Expr {
    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
    ) -> Result<(), ASTError> {
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
                        elem.resolve_names(name_scope, lambda_defs, session)?;
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
                    let lambda_id = NameScopeId::new_rand();

                    // TODO: `name_scope.push_names` after `ty.resolve_names`?
                    // -> dependent types?
                    name_scope.push_names(args, NameScopeKind::LambdaArg(lambda_id));

                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_names(name_scope, lambda_defs, session)?;
                        }
                    }

                    expr.resolve_names(name_scope, lambda_defs, session)?;
                    name_scope.pop_names();

                    let lambda_def = FuncDef::create_anonymous_function(
                        args.clone(),
                        *expr.clone(),
                        self.span,
                        lambda_id,
                        session,
                    );

                    if lambda_def.is_closure() {
                        todo!();
                    }

                    // No hash collision between programmer-defined names and newly generated name: the new ones have special characters
                    // But there may be collisions between newly generated ones -> TODO: what then?
                    if let Some(_) = lambda_defs.insert(lambda_def.name, lambda_def) {
                        todo!();
                    }

                    // *self = lambda_def

                    Ok(())
                },
                ValueKind::Block { defs, value, id } => {
                    name_scope.push_names(defs, NameScopeKind::Block(*id));

                    for (_, expr) in defs.iter_mut() {
                        expr.resolve_names(name_scope, lambda_defs, session)?;
                    }

                    value.resolve_names(name_scope, lambda_defs, session)?;
                    name_scope.pop_names();

                    Ok(())
                },
            },
            ExprKind::Prefix(_, operand) | ExprKind::Postfix(_, operand) => operand.resolve_names(name_scope, lambda_defs, session),
            ExprKind::Branch(cond, b1, b2) => {
                cond.resolve_names(name_scope, lambda_defs, session)?;
                b1.resolve_names(name_scope, lambda_defs, session)?;
                b2.resolve_names(name_scope, lambda_defs, session)?;

                Ok(())
            }
            ExprKind::Call(f, args) => {
                f.resolve_names(name_scope, lambda_defs, session)?;

                for arg in args.iter_mut() {
                    arg.resolve_names(name_scope, lambda_defs, session)?;
                }

                Ok(())
            }
            ExprKind::Infix(op, o1, o2) => match op {

                // `a.b.c` -> `a` has to be resolved, but the others shall not
                InfixOp::Path => o1.resolve_names(name_scope, lambda_defs, session),
                _ => {
                    o1.resolve_names(name_scope, lambda_defs, session)?;
                    o2.resolve_names(name_scope, lambda_defs, session)
                }
            },
        }
    }
}