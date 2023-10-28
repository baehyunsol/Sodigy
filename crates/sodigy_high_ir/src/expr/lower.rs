use super::Expr;
use sodigy_ast::{self as ast, ExprKind, ValueKind};

// it takes...
// all the names defined in this module
// all the `use`s defined in this module
// current name space
pub fn lower_ast_expr(e: &ast::Expr) -> Expr {
    match &e.kind {
        ExprKind::Value(v) => match &v {
            ValueKind::Identifier(id) => {
                // Find origin of id
                // if id is 'a' and there's `use a.b.c as a;`, apply that
                todo!()
            },
            ValueKind::Number(n) => todo!(),
            ValueKind::String { s, is_binary } => todo!(),
            ValueKind::Char(c) => todo!(),
            ValueKind::List(elems) => todo!(),
            ValueKind::Tuple(elems) => todo!(),
            ValueKind::Format(elems) => todo!(),
            ValueKind::Lambda {
                args, value,
            } => {
                // Push names defined in this lambda, then recurs
                todo!()
            },
            ValueKind::Scope(scope) => {
                // Push names defined in this scope, then recurs
                todo!()
            },
        },
        ExprKind::PrefixOp(op, expr) => todo!(),
        ExprKind::PostfixOp(op, expr) => todo!(),
        ExprKind::InfixOp(op, lhs, rhs) => todo!(),
        ExprKind::Path { pre, post } => todo!(),
        ExprKind::Call { functor, args } => todo!(),
        ExprKind::StructInit { struct_, init } => todo!(),
        ExprKind::Branch(arms) => {
            // Push names defined in the arms (if there's `if let`), then recurs
            todo!()
        },
        ExprKind::Match { value, arms } => {
            // Push names defined in the arms, then recurs
            todo!()
        },
    }
}
