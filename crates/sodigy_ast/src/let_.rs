use crate::{
    ArgDef,
    expr::Expr,
    FieldDef,
    GenericDef,
    IdentWithSpan,
    pattern::Pattern,
    TypeDef,
    VariantDef,
    stmt::Attribute,
};
use sodigy_uid::Uid;

mod fmt;

#[derive(Clone, Debug)]
pub struct Let {
    pub kind: LetKind,

    // if it's scoped-let, the attributes are here
    // if it's top-level-let, the attributes are in session.stmts
    pub attributes: Vec<Attribute>,
}

impl Let {
    pub fn pattern(pattern: Pattern, expr: Expr, attributes: Vec<Attribute>) -> Self {
        Let {
            kind: LetKind::Pattern(pattern, expr),
            attributes,
        }
    }

    pub fn def(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        args: Option<Vec<ArgDef>>,
        return_ty: Option<TypeDef>,
        return_val: Expr,
        attributes: Vec<Attribute>,
    ) -> Self {
        if let Some(args) = args {
            Let {
                kind: LetKind::Callable {
                    name, generics,
                    args, return_ty,
                    return_val,
                    uid: Uid::new_def(),
                },
                attributes,
            }
        }

        else {
            Let {
                kind: LetKind::Incallable {
                    name, generics,
                    return_ty, return_val,
                    uid: Uid::new_def(),
                },
                attributes,
            }
        }
    }

    pub fn enum_(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        variants: Vec<VariantDef>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Let {
            kind: LetKind::Enum {
                name,
                generics,
                variants,
                uid: Uid::new_enum(),
            },
            attributes,
        }
    }

    pub fn struct_(
        name: IdentWithSpan,
        generics: Vec<GenericDef>,
        fields: Vec<FieldDef>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Let {
            kind: LetKind::Struct {
                name,
                generics,
                fields,
                uid: Uid::new_struct(),
            },
            attributes,
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

#[derive(Clone, Debug)]
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
