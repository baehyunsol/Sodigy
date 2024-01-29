use crate::expr::Expr;
use crate::ty::Type;
use sodigy_ast::IdentWithSpan;
use sodigy_uid::Uid;

mod lower;

pub struct Def {
    name: IdentWithSpan,
    args: Option<Vec<Arg>>,
    pub(crate) return_ty: Type,
    pub(crate) return_val: Expr,
    uid: Uid,
    // TODO: and many more fields...
}

pub struct Arg {
    name: IdentWithSpan,  // TODO: do we need this?
    ty: Type,
}
