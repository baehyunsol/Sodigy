use crate::Punct;

#[derive(Clone, Copy, Debug)]
pub enum PostfixOp {
    Range,
    QuestionMark,
}

impl TryFrom<Punct> for PostfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<PostfixOp, ()> {
        match p {
            Punct::DotDot => Ok(PostfixOp::Range),
            Punct::QuestionMark => Ok(PostfixOp::QuestionMark),
            // Do not use a wildcard!
            Punct::Add | Punct::Sub | Punct::Mul |
            Punct::Div | Punct::Rem | Punct::Colon |
            Punct::Semicolon | Punct::Lt | Punct::Assign |
            Punct::Gt | Punct::Comma | Punct::Dot |
            Punct::Shl | Punct::Shr | Punct::Eq |
            Punct::Leq | Punct::Neq | Punct::Geq |
            Punct::Arrow => Err(()),
        }
    }
}
