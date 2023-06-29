use super::{Delimiter, Keyword, OpToken, Token};
use crate::session::LocalParseSession;
use hmath::Ratio;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Number(Ratio),
    String(u32),

    // It doesn't care how the inside looks like. It only guarantees that the opening and closing are properly matched.
    List(Delimiter, Vec<Box<Token>>),
    Identifier(u32),

    // True, False, None
    Keyword(Keyword),

    Operator(OpToken)
}

impl TokenKind {

    // preview of this token_kind for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            TokenKind::Number(_) => "Number".to_string(),
            TokenKind::String(_) => "String Literal".to_string(),
            TokenKind::List(delim, _) => match delim {
                Delimiter::Parenthesis => "(...)",
                Delimiter::Brace => "{...}",
                Delimiter::Bracket => "[...]",
            }.to_string(),
            TokenKind::Identifier(i) => format!(
                "Identifier: `{}`",
                String::from_utf8_lossy(&session.get_string_from_index(*i).unwrap_or(vec![b'?'; 3])).to_string()
            ),
            TokenKind::Keyword(k) => format!("Keyword: `{}`", k.render_err()),
            TokenKind::Operator(op) => format!("Special Character: `{}`", op.render_err())
        }
    }

}