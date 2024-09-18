use crate::{
    DottedNames,
    Expr,
    IdentWithSpan,
    let_::Let,
    TypeDef,
    utils::merge_dotted_names,
};
use sodigy_attribute::{Attribute, Decorator};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod fmt;

pub struct Stmt {
    pub kind: StmtKind,
    pub span: SpanRange,
}

pub enum StmtKind {
    Module(IdentWithSpan, Uid),
    Import(Import),
    Let(Let),
    Decorator(Decorator<Expr>),

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

#[derive(Clone, Debug)]
pub struct VariantDef {
    pub name: IdentWithSpan,
    pub args: VariantKind,
    pub attributes: Vec<Attribute<Expr>>,
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
    pub attributes: Vec<Attribute<Expr>>,
}
