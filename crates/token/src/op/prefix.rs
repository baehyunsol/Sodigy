use crate::Punct;

#[derive(Clone, Copy, Debug)]
pub enum PrefixOp {
    Not,
    Neg,
}

impl TryFrom<Punct> for PrefixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<PrefixOp, ()> {
        match p {
            Punct::Sub => Ok(PrefixOp::Neg),
            Punct::Factorial => Ok(PrefixOp::Not),
            // Do not use a wildcard!
            Punct::Add | Punct::Mul | Punct::Div | Punct::Rem |
            Punct::Colon | Punct::Semicolon |
            Punct::Lt | Punct::Assign | Punct::Gt |
            Punct::Comma | Punct::Dot | Punct::At |
            Punct::Factorial | Punct::QuestionMark | Punct::Dollar |
            Punct::And | Punct::Or | Punct::AndAnd | Punct::OrOr |
            Punct::Shl | Punct::Shr | Punct::Eq |
            Punct::Leq | Punct::Neq | Punct::Geq |
            Punct::Concat | Punct::Arrow |
            Punct::DotDot | Punct::DotDotEq |
            Punct::ReturnType => Err(()),
        }
    }
}
