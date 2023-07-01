use crate::session::InternedString;
use crate::span::Span;

mod list;
mod kind;

pub use list::TokenList;
pub use kind::TokenKind;

#[derive(Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind
}

impl Token {

    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    pub fn unwrap_identifier(&self) -> InternedString {
        self.kind.unwrap_identifier()
    }

}

impl PartialEq for Token {

    fn eq(&self, other: &Token) -> bool {
        self.kind == other.kind
    }

}

#[derive(Copy, Clone, PartialEq)]
pub enum Delimiter {
    Parenthesis,  // ()
    Bracket,  // []
    Brace  // {}
}

impl Delimiter {

    pub fn from(c: u8) -> Self {

        if c == b'(' {
            Delimiter::Parenthesis
        }

        else if c == b'{' {
            Delimiter::Brace
        }

        else if c == b'[' {
            Delimiter::Bracket
        }

        else {
            unreachable!("Interal Compiler Error 335FA8A: {c}")
        }

    }

    pub fn start(&self) -> u8 {
        match self {
            Delimiter::Parenthesis => b'(',
            Delimiter::Bracket => b'[',
            Delimiter::Brace => b'{',
        }
    }

    pub fn end(&self) -> u8 {
        match self {
            Delimiter::Parenthesis => b')',
            Delimiter::Bracket => b']',
            Delimiter::Brace => b'}',
        }
    }

}

#[derive(Copy, Clone, PartialEq)]
pub enum Keyword {
    If, Else, Def, Use,
}

impl Keyword {

    // preview of this keyword for error messages
    pub fn render_err(&self) -> String {
        match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Def => "def",
            Keyword::Use => "use",
        }.to_string()
    }

}

// It doesn't care whether it's binary or unary,
// but it cares whether it's `<` or `<=`
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpToken {
    Add, Sub, Mul, Div, Rem,
    Not,     // `!`
    Concat,  // `<>`
    Assign,  // `=`
    Eq, Lt, Gt, Ne, Le, Ge,
    And, AndAnd, Or, OrOr,
    Comma, Dot, Colon, SemiColon,
    DotDot,

    // below 4 are not used by lexer, but by `get_first_token`
    OpeningSquareBracket,
    ClosingSquareBracket,
    OpeningCurlyBrace,
    ClosingCurlyBrace,
}

impl OpToken {

    // preview of this token for error messages
    pub fn render_err(&self) -> String {
        match self {
            OpToken::Add => "+",
            OpToken::Sub => "-",
            OpToken::Mul => "*",
            OpToken::Div => "/",
            OpToken::Rem => "%",
            OpToken::Not => "!",
            OpToken::Concat => "<>",
            OpToken::Eq => "==",
            OpToken::Lt => "<",
            OpToken::Gt => ">",
            OpToken::Ne => "!=",
            OpToken::Le => "<=",
            OpToken::Ge => ">=",
            OpToken::And => "&",
            OpToken::AndAnd => "&&",
            OpToken::Or => "|",
            OpToken::OrOr => "||",
            OpToken::Comma => ",",
            OpToken::Dot => ".",
            OpToken::Colon => ":",
            OpToken::SemiColon => ";",
            OpToken::DotDot => "..",
            OpToken::Assign => "=",
            OpToken::OpeningSquareBracket => "[",
            OpToken::ClosingSquareBracket => "]",
            OpToken::OpeningCurlyBrace => "{",
            OpToken::ClosingCurlyBrace => "}",
        }.to_string()
    }

}