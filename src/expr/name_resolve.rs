use super::{Expr, ExprKind};
use crate::ast::{ASTError, NameScope};
use crate::expr::InfixOp;
use crate::stmt::ArgDef;
use crate::value::ValueKind;

impl Expr {
    pub fn resolve_names(&mut self, name_scope: &mut NameScope) -> Result<(), ASTError> {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_) | ValueKind::Real(_) | ValueKind::String(_) | ValueKind::Bytes(_) => {
                    return Ok(());
                },
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for elem in elements.iter_mut() {
                        elem.resolve_names(name_scope)?;
                    }

                    Ok(())
                },
                ValueKind::Identifier(id) => match name_scope.search_name(*id) {
                    Ok(None) => Ok(()),
                    Ok(Some(alias)) => {
                        self.kind = alias.to_path();

                        Ok(())
                    },
                    Err(()) => Err(ASTError::no_def(*id, self.span, name_scope.clone())),
                },
                ValueKind::Lambda(args, expr) => {

                    // TODO: `name_scope.push_names` after `ty.resolve_names`?
                    // -> dependent types?
                    name_scope.push_names(args);

                    for ArgDef { ty, .. } in args.iter_mut() {
                        ty.resolve_names(name_scope)?;
                    }

                    expr.resolve_names(name_scope)?;
                    name_scope.pop_names();

                    Ok(())
                },
                ValueKind::Block { defs, value } => {
                    name_scope.push_names(defs);

                    for (_, expr) in defs.iter_mut() {
                        expr.resolve_names(name_scope)?;
                    }

                    value.resolve_names(name_scope)?;
                    name_scope.pop_names();

                    Ok(())
                },
            },
            ExprKind::Prefix(_, operand) | ExprKind::Postfix(_, operand) => operand.resolve_names(name_scope),
            ExprKind::Branch(cond, b1, b2) => {
                cond.resolve_names(name_scope)?;
                b1.resolve_names(name_scope)?;
                b2.resolve_names(name_scope)?;

                Ok(())
            }
            ExprKind::Call(f, args) => {
                f.resolve_names(name_scope)?;

                for arg in args.iter_mut() {
                    arg.resolve_names(name_scope)?;
                }

                Ok(())
            }
            ExprKind::Infix(op, o1, o2) => match op {

                // `a.b.c` -> `a` has to be resolved, but the others shall not
                InfixOp::Path => {
                    o1.resolve_names(name_scope)?;

                    Ok(())
                },
                _ => {
                    o1.resolve_names(name_scope)?;
                    o2.resolve_names(name_scope)?;

                    Ok(())
                }
            },
        }
    }
}