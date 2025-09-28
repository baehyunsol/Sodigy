use crate::Punct;

#[derive(Clone, Copy, Debug)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Lt,
    Eq,
    Gt,
    Leq,
    Neq,
    Geq,
    Index,
}

impl TryFrom<Punct> for InfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<InfixOp, ()> {
        match p {
            Punct::Add => Ok(InfixOp::Add),
            Punct::Sub => Ok(InfixOp::Sub),
            Punct::Mul => Ok(InfixOp::Mul),
            Punct::Div => Ok(InfixOp::Div),
            Punct::Rem => Ok(InfixOp::Rem),
            Punct::Shl => Ok(InfixOp::Shl),
            Punct::Shr => Ok(InfixOp::Shr),
            Punct::Lt => Ok(InfixOp::Lt),
            Punct::Eq => Ok(InfixOp::Eq),
            Punct::Gt => Ok(InfixOp::Gt),
            Punct::Leq => Ok(InfixOp::Leq),
            Punct::Neq => Ok(InfixOp::Neq),
            Punct::Geq => Ok(InfixOp::Geq),
            // Do not use a wildcard!
            Punct::Colon | Punct::Semicolon | Punct::Assign |
            Punct::Comma | Punct::Dot | Punct::QuestionMark |
            Punct::DotDot | Punct::Arrow => Err(()),
        }
    }
}
