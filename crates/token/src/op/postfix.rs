use crate::Punct;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PostfixOp {
    Range { inclusive: bool },
    QuestionMark,
}

impl PostfixOp {
    pub fn get_def_lang_item(&self) -> &'static str {
        match self {
            PostfixOp::Range { inclusive: true } => "op.inclusive_post_range",
            PostfixOp::Range { inclusive: false } => "op.exclusive_post_range",
            PostfixOp::QuestionMark => "op.question_mark",
        }
    }

    pub fn get_generic_lang_items(&self) -> Vec<&'static str> {
        match self {
            PostfixOp::Range { inclusive: true } => vec!["op.inclusive_range.generic.0", "op.inclusive_range.generic.1"],
            PostfixOp::Range { inclusive: false } => vec!["op.exclusive_range.generic.0", "op.exclusive_range.generic.1"],
            PostfixOp::QuestionMark => vec!["op.question_mark.generic.0", "op.question_mark.generic.1"],
        }
    }

    pub fn render_error(&self) -> &'static str {
        match self {
            PostfixOp::Range { inclusive: true } => "..=",
            PostfixOp::Range { inclusive: false } => "..",
            PostfixOp::QuestionMark => "?",
        }
    }
}

impl TryFrom<Punct> for PostfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<PostfixOp, ()> {
        match p {
            Punct::DotDot => Ok(PostfixOp::Range { inclusive: false }),
            Punct::DotDotEq => Ok(PostfixOp::Range { inclusive: true }),
            Punct::QuestionMark => Ok(PostfixOp::QuestionMark),
            // Do not use a wildcard!
            Punct::Add | Punct::Sub | Punct::Mul |
            Punct::Div | Punct::Rem | Punct::Colon | Punct::Semicolon |
            Punct::Lt | Punct::Assign | Punct::Gt |
            Punct::Comma | Punct::Dot | Punct::At |
            Punct::Dollar | Punct::Factorial |
            Punct::And | Punct::Or | Punct::Xor |
            Punct::AndAnd | Punct::OrOr |
            Punct::Shl | Punct::Shr | Punct::Eq |
            Punct::Leq | Punct::Neq | Punct::Geq |
            Punct::Concat | Punct::Append | Punct::Prepend |
            Punct::Arrow | Punct::ReturnType | Punct::Pipeline => Err(()),
        }
    }
}
