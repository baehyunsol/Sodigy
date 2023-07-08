use super::{Expr, ExprKind};
use crate::ast::{ASTError, NameScope};
use crate::value::ValueKind;

impl Expr {
    pub fn resolve_names(&mut self, name_scope: &mut NameScope) -> Result<(), ASTError> {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_) | ValueKind::Real(_) | ValueKind::String(_) => {
                    return Ok(());
                },
                ValueKind::List(elements) | ValueKind::Tuple(elements) => {
                    for elem in elements.iter_mut() {
                        elem.resolve_names(name_scope)?;
                    }

                    Ok(())
                },
                ValueKind::Identifier(id) => match name_scope.search_name(*id) {
                    Ok(None) => Ok(()),
                    Ok(Some(alias)) => {
                        todo!();

                        Ok(())
                    },
                    Err(()) => Err(ASTError::no_def(*id, self.span, name_scope.clone())),
                },
                ValueKind::Lambda(args, expr) => todo!(),
                ValueKind::Block { defs, value } => todo!(),
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
            ExprKind::Infix(op, o1, o2) => todo!(),  // InfixOp::Path must be taken a special care
        }
    }
}