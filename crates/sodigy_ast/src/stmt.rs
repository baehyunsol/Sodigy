use crate::{utils::merge_dotted_names, ArgDef, DottedNames, Expr, GenericDef, IdentWithSpan, TypeDef};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

pub struct Stmt {
    pub kind: StmtKind,

    // `span` includes the entire definition
    // e.g. from a keyword `def` to an expression for a function
    pub span: SpanRange,
}

pub enum StmtKind {
    Func(FuncDef),
    Module(IdentWithSpan, Uid),
    Import(Import),
    Enum(EnumDef),
    Struct(StructDef),
    Decorator(Decorator),

    // consecutive doc comments are not merged yet
    DocComment(InternedString),
}

impl StmtKind {
    pub fn get_id(&self) -> Option<&IdentWithSpan> {
        match self {
            StmtKind::Func(func) => Some(&func.name),
            StmtKind::Module(m, _) => Some(m),
            StmtKind::Enum(en) => Some(&en.name),
            StmtKind::Struct(st) => Some(&st.name),
            _ => None,
        }
    }

    pub fn get_uid(&self) -> Option<&Uid> {
        match self {
            StmtKind::Func(func) => Some(&func.uid),
            StmtKind::Enum(e) => Some(&e.uid),
            StmtKind::Struct(s) => Some(&s.uid),
            StmtKind::Module(_, id) => Some(id),
            _ => None,
        }
    }
}

pub struct FuncDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub args: Option<Vec<ArgDef>>,
    pub ret_type: Option<TypeDef>,
    pub ret_val: Expr,
    pub uid: Uid,
}

// `import a, b, c.d, e as f from x.y;`
pub struct Import {
    pub names: Vec<ImportedName>,
    pub from: Option<DottedNames>,
}

impl Import {
    // `import a.b.c as d;` -> buffer.push((d, [a, b, c]))
    pub fn unfold_alias(&self, buffer: &mut Vec<(IdentWithSpan, Vec<IdentWithSpan>)>) {
        let empty_vec = vec![];
        let prefix = self.from.as_ref().unwrap_or(&empty_vec);

        for name in self.names.iter() {
            buffer.push((
                *name.get_alias(),
                merge_dotted_names(&prefix, &name.name).iter().map(|i| *i).collect(),
            ));
        }
    }
}

pub struct ImportedName {
    pub name: DottedNames,
    pub alias: Option<IdentWithSpan>,
}

impl ImportedName {
    pub fn get_alias(&self) -> &IdentWithSpan {
        if let Some(id) = &self.alias {
            id
        }

        else {
            self.name.last().unwrap()
        }
    }
}

// attributes of enums and structs are collected later
// in ast level, it only collects attributes of variants and fields
pub enum Attribute {
    DocComment(String),
    Decorator(Decorator),
}

pub struct EnumDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub variants: Vec<VariantDef>,
    pub uid: Uid,
}

pub enum VariantKind {
    Empty,
    Tuple(Vec<TypeDef>),
    Struct(Vec<FieldDef>),
}

pub struct VariantDef {
    pub name: IdentWithSpan,
    pub args: VariantKind,
    pub attributes: Vec<Attribute>,
}

pub struct StructDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub fields: Vec<FieldDef>,
    pub uid: Uid,
}

pub struct FieldDef {
    pub name: IdentWithSpan,
    pub ty: TypeDef,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone)]
pub struct Decorator {
    pub name: DottedNames,
    pub args: Option<Vec<Expr>>,
}
