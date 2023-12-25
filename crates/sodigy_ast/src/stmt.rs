use crate::{
    ArgDef,
    DottedNames,
    Expr,
    GenericDef,
    IdentWithSpan,
    let_::Let,
    TypeDef,
    utils::merge_dotted_names,
};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

pub struct Stmt {
    pub kind: StmtKind,
    pub span: SpanRange,
}

pub enum StmtKind {
    Module(IdentWithSpan, Uid),
    Import(Import),
    Let(Let),
    Decorator(Decorator),

    // consecutive doc comments are not merged yet
    DocComment(InternedString),
}

impl StmtKind {
    pub fn get_id(&self) -> Option<IdentWithSpan> {
        match self {
            StmtKind::Module(m, _) => Some(*m),
            StmtKind::Let(l) => l.get_id(),
            StmtKind::Import(_)
            | StmtKind::Decorator(_)
            | StmtKind::DocComment(_) => None,
        }
    }

    pub fn get_uid(&self) -> Option<Uid> {
        match self {
            StmtKind::Module(_, id) => Some(*id),
            StmtKind::Let(l) => l.get_uid(),
            StmtKind::Import(_)
            | StmtKind::Decorator(_)
            | StmtKind::DocComment(_) => None,
        }
    }
}

pub struct FuncDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub args: Option<Vec<ArgDef>>,
    pub return_ty: Option<TypeDef>,
    pub return_val: Expr,
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
#[derive(Clone, Debug)]
pub enum Attribute {
    DocComment(IdentWithSpan),
    Decorator(Decorator),
}

impl Attribute {
    pub fn span(&self) -> SpanRange {
        match self {
            Attribute::DocComment(iws) => *iws.span(),
            Attribute::Decorator(dec) => dec.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariantDef {
    pub name: IdentWithSpan,
    pub args: VariantKind,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub enum VariantKind {
    Empty,
    Tuple(Vec<TypeDef>),
    Struct(Vec<FieldDef>),
}

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub name: IdentWithSpan,
    pub ty: TypeDef,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct Decorator {
    pub name: DottedNames,
    pub args: Option<Vec<Expr>>,
}

impl Decorator {
    pub fn span(&self) -> SpanRange {
        let mut result = *self.name[0].span();

        for name in self.name[1..].iter() {
            result = result.merge(*name.span());
        }

        result
    }
}
