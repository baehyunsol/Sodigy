use super::Punct::{self, *};
use std::fmt;

impl fmt::Display for Punct {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                At => "@".to_string(),
                Add => "+".to_string(),
                Sub => "-".to_string(),
                Mul => "*".to_string(),
                Div => "/".to_string(),
                Rem => "%".to_string(),
                Not => "!".to_string(),
                Concat => "<>".to_string(),
                Assign => "=".to_string(),
                Eq => "==".to_string(),
                Lt => "<".to_string(),
                Gt => ">".to_string(),
                Ne => "!=".to_string(),
                Le => "<=".to_string(),
                Ge => ">=".to_string(),
                And => "&".to_string(),
                AndAnd => "&&".to_string(),
                Or => "|".to_string(),
                OrOr => "||".to_string(),
                Comma => ",".to_string(),
                Dot => ".".to_string(),
                Colon => ":".to_string(),
                SemiColon => ";".to_string(),
                DotDot => "..".to_string(),
                Backslash => "\\".to_string(),
                Dollar => "$".to_string(),
                Backtick => "`".to_string(),
                QuestionMark => "?".to_string(),
                InclusiveRange => "..~".to_string(),
                RArrow => "=>".to_string(),
                Append => "<+".to_string(),
                Prepend => "+>".to_string(),
                FieldModifier(id) => format!("`{id}"),
            },
        )
    }
}
