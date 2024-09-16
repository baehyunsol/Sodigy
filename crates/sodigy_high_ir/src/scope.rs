use crate::Type;
use crate::expr::Expr;
use crate::pattern::Pattern;
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

mod endec;

#[derive(Clone)]
pub struct Scope {
    // used later for type-checking
    pub original_patterns: Vec<(Pattern, Expr)>,

    pub lets: Vec<ScopedLet>,
    pub value: Box<Expr>,
    pub uid: Uid,
}

#[derive(Clone)]
pub struct ScopedLet {
    pub name: IdentWithSpan,
    pub value: Expr,
    pub ty: Option<Type>,

    // the compiler generates tmp local defs during the compilation
    pub is_real: bool,
}

impl ScopedLet {
    pub fn try_new(name: IdentWithSpan, value: Result<Expr, ()>, ty: Option<Result<Type, ()>>, is_real: bool) -> Option<Self> {
        match (&value, &ty) {
            (Ok(_), Some(Ok(_))) => Some(ScopedLet {
                name, value: value.unwrap(),
                ty: ty.map(|ty| ty.unwrap()), is_real,
            }),
            (Ok(_), None) => Some(ScopedLet {
                name, value: value.unwrap(),
                ty: None, is_real,
            }),
            _ => None,
        }
    }
}
