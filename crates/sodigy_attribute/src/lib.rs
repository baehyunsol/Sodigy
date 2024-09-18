#![deny(unused_imports)]

use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;

mod endec;
mod fmt;

pub type DottedNames = Vec<IdentWithSpan>;

#[derive(Clone, Debug)]
pub enum Attribute<Expr> {
    DocComment(IdentWithSpan),
    Decorator(Decorator<Expr>),
}

impl<Expr> Attribute<Expr> {
    pub fn span(&self) -> SpanRange {
        match self {
            Attribute::DocComment(iws) => *iws.span(),
            Attribute::Decorator(dec) => dec.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decorator<Expr> {
    pub name: DottedNames,
    pub args: Option<Vec<Expr>>,
}

impl<Expr> Decorator<Expr> {
    pub fn span(&self) -> SpanRange {
        let mut result = *self.name[0].span();

        for name in self.name[1..].iter() {
            result = result.merge(*name.span());
        }

        result
    }
}
