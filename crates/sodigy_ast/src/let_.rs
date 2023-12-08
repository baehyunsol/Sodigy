use crate::{
    ArgDef,
    expr::Expr,
    FieldDef,
    GenericDef,
    IdentWithSpan,
    pattern::Pattern,
    TypeDef,
    VariantDef,
};
use sodigy_uid::Uid;

#[derive(Clone)]
pub struct Let {
    pub kind: LetKind,

    // TODO: why do we need this struct?
}

impl Let {
    pub fn pattern(pattern: Pattern, expr: Expr) -> Self {
        Let {
            kind: LetKind::Pattern(pattern, expr),
        }
    }

    pub fn def(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        args: Option<Vec<ArgDef>>,
        return_ty: Option<TypeDef>,
        return_val: Expr,
    ) -> Self {
        if let Some(args) = args {
            Let {
                kind: LetKind::Callable {
                    name, generics,
                    args, return_ty,
                    return_val,
                    uid: Uid::new_def(),
                },
            }
        }

        else {
            Let {
                kind: LetKind::Incallable {
                    name, generics,
                    return_ty, return_val,
                    uid: Uid::new_def(),
                },
            }
        }
    }

    pub fn enum_(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        variants: Vec<VariantDef>,
    ) -> Self {
        Let {
            kind: LetKind::Enum {
                name,
                generics,
                variants,
                uid: Uid::new_enum(),
            },
        }
    }

    pub fn struct_(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        fields: Vec<FieldDef>,
    ) -> Self {
        Let {
            kind: LetKind::Struct {
                name,
                generics,
                fields,
                uid: Uid::new_struct(),
            },
        }
    }

    pub fn get_id(&self) -> Option<IdentWithSpan> {
        match &self.kind {
            LetKind::Incallable { name, .. }
            | LetKind::Callable { name, .. }
            | LetKind::Enum { name, .. }
            | LetKind::Struct { name, .. } => Some(*name),
            _ => None,
        }
    }

    pub fn get_uid(&self) -> Option<Uid> {
        match &self.kind {
            LetKind::Incallable { uid, .. }
            | LetKind::Callable { uid, .. }
            | LetKind::Enum { uid, .. }
            | LetKind::Struct { uid, .. } => Some(*uid),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum LetKind {
    Pattern(Pattern, Expr),
    Incallable {
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        return_ty: Option<TypeDef>,
        return_val: Expr,
        uid: Uid,
    },
    Callable {
        name: IdentWithSpan,
        args: Vec<ArgDef>,
        generics: Vec<GenericDef>,
        return_ty: Option<TypeDef>,
        return_val: Expr,
        uid: Uid,
    },
    Enum {
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        variants: Vec<VariantDef>,
        uid: Uid,
    },
    Struct {
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        fields: Vec<FieldDef>,
        uid: Uid,
    }
}
