use super::super::{AST, ASTError, NameOrigin};
use crate::expr::{Expr, ExprKind, MatchBranch};
use crate::iter_mut_exprs_in_ast;
use crate::session::LocalParseSession;
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};

/*
```
{
    a = \{n, if n > 0 { a(n - 1) } else { 0 }};

    a
}
```

The name resolver thinks that `a` is a closure, because it's referencing `a`, which is not in the lambda's name scope.
But it's obvious that `a` is not a closure. This pass visits all the exprs, finds such cases, and fixes them.
It also deals with mutually recursive cases

1. if it finds `Call(@@LAMBDA_ABCDEF, a)`, which is a closure, it checks whether all of the arguments (captured vars) are functors
2. if so, it changes `Call(@@LAMBDA_ABCDEF, a)` to `@@LAMBDA_ABCDEF` and modify the def of `@@LAMBDA_ABCDEF` in AST

This pass must be called after name_resolve and before block_clean_up because,
1. name_resolve creates lambda definitions
2. block_clean_up will reject recursive lambda functions (unless this pass) because they reject recursive block defs
*/

iter_mut_exprs_in_ast!(resolve_recursive_funcs_in_block);

impl Expr {
    pub fn resolve_recursive_funcs_in_block(&mut self, session: &mut LocalParseSession) -> Result<(), ASTError> {

        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_, _)
                | ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for element in elements.iter_mut() {
                        element.resolve_recursive_funcs_in_block(session)?;
                    }
                },
                ValueKind::Closure(f, captured_variables) => {
                    for (name, origin) in captured_variables.iter() {
                        match origin {
                            NameOrigin::BlockDef(id) => {
                                // let's check whether it's a closure
                                // if this value is a const-lambda (which looks like a closure)
                            }
                            _ => {
                                // definitely a closure
                            }
                        }
                    }
                },
                ValueKind::Lambda(args, val) => {
                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_recursive_funcs_in_block(session)?;
                        }
                    }

                    val.resolve_recursive_funcs_in_block(session)?;
                },
                ValueKind::Block { defs, value, .. } => {

                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        value.resolve_recursive_funcs_in_block(session)?;

                        if let Some(ty) = ty {
                            ty.resolve_recursive_funcs_in_block(session)?;
                        }
                    }

                    value.resolve_recursive_funcs_in_block(session)?;
                },
            },
            ExprKind::Prefix(_, v) => v.resolve_recursive_funcs_in_block(session)?,
            ExprKind::Postfix(_, v) => v.resolve_recursive_funcs_in_block(session)?,
            ExprKind::Infix(_, v1, v2) => {
                v1.resolve_recursive_funcs_in_block(session)?;
                v2.resolve_recursive_funcs_in_block(session)?;
            },
            ExprKind::Match(value, branches, _) => {
                value.resolve_recursive_funcs_in_block(session)?;

                for MatchBranch { value, .. } in branches.iter_mut() {
                    value.resolve_recursive_funcs_in_block(session)?;
                }
            },
            ExprKind::Branch(c, t, f) => {
                c.resolve_recursive_funcs_in_block(session)?;
                t.resolve_recursive_funcs_in_block(session)?;
                f.resolve_recursive_funcs_in_block(session)?;
            },
            ExprKind::Call(f, args) => {
                f.resolve_recursive_funcs_in_block(session)?;

                for arg in args.iter_mut() {
                    arg.resolve_recursive_funcs_in_block(session)?;
                }
            }
        }

        // though it doesn't return any error, it's return type is `Result`, due to the macro
        Ok(())
    }
}
