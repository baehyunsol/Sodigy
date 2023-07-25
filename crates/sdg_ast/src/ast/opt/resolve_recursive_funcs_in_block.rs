use super::super::{AST, ASTError, NameOrigin};
use crate::expr::{Expr, ExprKind};
use crate::iter_mut_exprs_in_ast;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{ArgDef, Decorator, LAMBDA_FUNC_PREFIX};
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
                ValueKind::Lambda(args, val) => {
                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_recursive_funcs_in_block(session)?;
                        }
                    }

                    val.resolve_recursive_funcs_in_block(session)?;
                },
                ValueKind::Block { defs, value, id } => {

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

                // if so, it's a closure
                if f.is_lambda_function_name(session) {
                    let functors = args.iter().filter(
                        |arg| is_definitely_functor(arg, /* some kind of context*/)
                    ).map(
                        |arg| arg.clone()
                    ).collect::<Vec<_>>();

                    if functors.len() == args.len() {
                        // it's not a closure any more
                    }

                    else if functors.len() > 0 {
                        // remove arg: optimization anyway
                    }

                    // TODO: we have to change the shap of `f` in AST
                }
            }
        }

        // though it doesn't return any error, it's return type is `Result`, due to the macro
        Ok(())
    }

    fn is_lambda_function_name(&self, session: &LocalParseSession) -> bool {
        match self.kind {
            ExprKind::Value(ValueKind::Identifier(id, NameOrigin::Local)) if id.is_lambda_function_name(session) => true,
            _ => false,
        }
    }
}

impl InternedString {
    pub fn is_lambda_function_name(&self, session: &LocalParseSession) -> bool {
        session.unintern_string(*self).starts_with(LAMBDA_FUNC_PREFIX.as_bytes())
    }
}

fn is_definitely_functor(arg: &Expr) -> bool {
    todo!()
}
