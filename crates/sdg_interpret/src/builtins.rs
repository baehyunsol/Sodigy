use sdg_ast::{Expr, ExprKind, utils, ValueKind};

// The only way to add methods to `Expr`
pub trait BuiltIns {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool;
    fn is_subtype_of(&self, other: &Expr) -> bool;

    /// it returns a Sodigy String object.\
    /// it's the actual `.to_string()` method inside Sodigy language.
    fn to_string(&self) -> Expr;
    fn to_rust_string(&self) -> Option<String>;
}

impl BuiltIns for Expr {
    fn is_true(&self) -> bool {
        todo!()
    }

    fn is_false(&self) -> bool {
        todo!()
    }

    fn is_subtype_of(&self, other: &Expr) -> bool {
        todo!()
    }

    fn to_string(&self) -> Expr {
        todo!()
    }

    fn to_rust_string(&self) -> Option<String> {
        match &self.kind {
            ExprKind::Value(ValueKind::String(s)) => utils::v32_to_string(s).ok(),
            _ => None,
        }
    }
}
