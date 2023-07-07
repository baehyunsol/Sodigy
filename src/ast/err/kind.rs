use crate::session::InternedString;

pub enum ASTErrorKind {
    MultipleDef(InternedString),
    UndefinedSymbol(InternedString),
    DecoratorOnUse,
}