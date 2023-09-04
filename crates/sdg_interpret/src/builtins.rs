use sdg_ast::{Expr, ExprKind, utils, ValueKind};
use sdg_uid::prelude;

// The only way to add methods to `Expr`
pub trait BuiltIns {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool;
    fn is_subtype_of(&self, other: &Expr) -> bool;
    fn is_type(&self) -> bool;

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

    fn is_type(&self) -> bool {
        match self.kind {
            ExprKind::Value(ValueKind::Object(id)) if id == prelude::type_() => true,
            _ => false,
        }
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
