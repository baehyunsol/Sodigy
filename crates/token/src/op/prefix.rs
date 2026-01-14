use crate::Punct;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PrefixOp {
    Not,
    Neg,
    Range { inclusive: bool },
}

impl PrefixOp {
    pub fn get_def_lang_item(&self) -> &'static str {
        match self {
            PrefixOp::Not => "op.not",
            PrefixOp::Neg => "op.neg",
            PrefixOp::Range { inclusive: true } => "op.inclusive_pre_range",
            PrefixOp::Range { inclusive: false } => "op.exclusive_pre_range",
        }
    }

    pub fn get_generic_lang_items(&self) -> Vec<&'static str> {
        match self {
            PrefixOp::Not => vec![],
            PrefixOp::Neg => vec!["op.neg.generic.0"],
            PrefixOp::Range { inclusive: true } => vec!["op.inclusive_pre_range.generic.0"],
            PrefixOp::Range { inclusive: false } => vec!["op.exclusive_pre_range.generic.0"],
        }
    }

    // Used when generating error messages.
    pub fn render_error(&self) -> &'static str {
        match self {
            PrefixOp::Not => "!",
            PrefixOp::Neg => "-",
            PrefixOp::Range { inclusive: true } => "..=",
            PrefixOp::Range { inclusive: false } => "..",
        }
    }
}

impl TryFrom<Punct> for PrefixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<PrefixOp, ()> {
        match p {
            Punct::Sub => Ok(PrefixOp::Neg),
            Punct::Factorial => Ok(PrefixOp::Not),
            Punct::DotDotEq => Ok(PrefixOp::Range { inclusive: true }),
            Punct::DotDot => Ok(PrefixOp::Range { inclusive: false }),
            // Do not use a wildcard!
            Punct::Add | Punct::Mul | Punct::Div | Punct::Rem |
            Punct::Colon | Punct::Semicolon |
            Punct::Lt | Punct::Assign | Punct::Gt |
            Punct::Comma | Punct::Dot | Punct::At |
            Punct::QuestionMark | Punct::Dollar |
            Punct::And | Punct::Or | Punct::Xor |
            Punct::AndAnd | Punct::OrOr |
            Punct::Shl | Punct::Shr | Punct::Eq |
            Punct::Leq | Punct::Neq | Punct::Geq |
            Punct::Concat | Punct::Append | Punct::Prepend |
            Punct::Arrow | Punct::ReturnType | Punct::Pipeline => Err(()),
        }
    }
}
