use super::{InfixOp, PostfixOp, PrefixOp};
use std::fmt;

impl fmt::Display for InfixOp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            InfixOp::Add => String::from("+"),
            InfixOp::Sub => String::from("-"),
            InfixOp::Mul => String::from("*"),
            InfixOp::Div => String::from("/"),
            InfixOp::Rem => String::from("%"),
            InfixOp::Eq => String::from("=="),
            InfixOp::Gt => String::from(">"),
            InfixOp::Lt => String::from("<"),
            InfixOp::Ne => String::from("!="),
            InfixOp::Ge => String::from(">="),
            InfixOp::Le => String::from("<="),
            InfixOp::BitwiseAnd => String::from("&"),
            InfixOp::BitwiseOr => String::from("|"),
            InfixOp::LogicalAnd => String::from("&&"),
            InfixOp::LogicalOr => String::from("||"),
            InfixOp::Xor => String::from("^"),
            InfixOp::ShiftRight => String::from(">>"),
            InfixOp::ShiftLeft => String::from("<<"),
            InfixOp::Index => String::from("[]"),
            InfixOp::Concat => String::from("<>"),
            InfixOp::Range => String::from(".."),
            InfixOp::InclusiveRange => String::from("..~"),
            InfixOp::FieldModifier(ids) => ids.iter().map(|id| format!("`{}", id.id())).collect::<Vec<_>>().join(" "),
        };

        write!(fmt, "{s}")
    }
}

impl fmt::Display for PrefixOp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            PrefixOp::Not => "!",
            PrefixOp::Neg => "-",
        };

        write!(fmt, "{s}")
    }
}

impl fmt::Display for PostfixOp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            PostfixOp::Range => "..",
            PostfixOp::QuestionMark => "?",
        };

        write!(fmt, "{s}")
    }
}
