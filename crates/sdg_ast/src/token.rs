use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use hmath::Ratio;

mod kind;
mod list;
mod macros;

pub use kind::TokenKind;
pub use list::TokenList;

#[derive(Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    pub fn unwrap_identifier(&self) -> InternedString {
        self.kind.unwrap_identifier()
    }

    pub fn is_number(&self) -> bool {
        self.kind.is_number()
    }

    pub fn unwrap_number(&self) -> Ratio {
        self.kind.unwrap_number()
    }

    pub fn unwrap_delimiter(&self) -> Delimiter {
        self.kind.unwrap_delimiter()
    }

    pub fn is_string(&self) -> bool {
        self.kind.is_string()
    }

    pub fn unwrap_string(&self) -> &Vec<u32> {
        self.kind.unwrap_string()
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        match &self.kind {
            TokenKind::Number(n) => format!("{n}"),
            TokenKind::String(s) => format!(
                "{:?}",
                s.iter().map(
                    |n| char::from_u32(*n).unwrap()
                ).collect::<String>()
            ),
            TokenKind::Bytes(b) => format!("Bytes({b:?})"),
            TokenKind::FormattedString(s) => format!(
                "Format({})",
                s.iter().map(
                    |s| format!(
                        "[{}]",
                        s.iter().map(
                            |s| s.dump(session)
                        ).collect::<Vec<String>>().join(", ")
                    )
                ).collect::<Vec<String>>().join(", "),
            ),
            TokenKind::List(delim, elements) => format!(
                "{}{}{}",
                delim.start() as char,
                elements.iter().map(
                    |e| e.dump(session)
                ).collect::<Vec<String>>().join(", "),
                delim.end() as char,
            ),
            TokenKind::Identifier(id) => id.to_string(session),
            TokenKind::Operator(op) => op.render_err(),
            TokenKind::Keyword(k) => k.render_err(),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        self.kind == other.kind
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Delimiter {
    Parenthesis, // ()
    Bracket,     // []
    Brace,       // {}
}

impl Delimiter {
    pub fn from(c: u8) -> Self {
        if c == b'(' {
            Delimiter::Parenthesis
        } else if c == b'{' {
            Delimiter::Brace
        } else if c == b'[' {
            Delimiter::Bracket
        } else {
            unreachable!("Internal Compiler Error 09D4C963D4F: {c}")
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

    pub fn opening_token_kind(&self) -> TokenKind {
        match self {
            Delimiter::Parenthesis => TokenKind::Operator(OpToken::OpeningParenthesis),
            Delimiter::Bracket => TokenKind::Operator(OpToken::OpeningSquareBracket),
            Delimiter::Brace => TokenKind::Operator(OpToken::OpeningCurlyBrace),
        }
    }

    pub fn closing_token_kind(&self) -> TokenKind {
        match self {
            Delimiter::Parenthesis => TokenKind::Operator(OpToken::ClosingParenthesis),
            Delimiter::Bracket => TokenKind::Operator(OpToken::ClosingSquareBracket),
            Delimiter::Brace => TokenKind::Operator(OpToken::ClosingCurlyBrace),
        }
    }
}

use std::fmt::{Error as FmtError, Formatter, Display};

impl Display for Delimiter {

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        write!(fmt, "{}", self.start() as char)
    }

}

#[derive(Copy, Clone, PartialEq)]
pub enum Keyword {
    If,
    Else,
    Def,
    Use,
    As,
    Let,
    Module,
    Match,
}

impl Keyword {
    // preview of this keyword for error messages
    pub fn render_err(&self) -> String {
        match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Def => "def",
            Keyword::Use => "use",
            Keyword::As => "as",
            Keyword::Let => "let",
            Keyword::Module => "module",
            Keyword::Match => "match",
        }.to_string()
    }
}

// It doesn't care whether it's binary or unary,
// but it cares whether it's `<` or `<=`
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpToken {
    At, // `@`
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Not,    // `!`
    Concat, // `<>`
    Assign, // `=`
    Eq,
    Lt,
    Gt,
    Ne,
    Le,
    Ge,
    And,
    AndAnd,
    Or,
    OrOr,
    Comma,
    Dot,
    Colon,
    SemiColon,
    DotDot,
    BackSlash,
    Dollar,
    BackTick,
    InclusiveRange,  // `..~`
    RArrow,   // `=>`
    Append,   // `<+`
    Prepend,  // `+>`

    // below 4 are not used by lexer, but by `get_first_token`
    OpeningSquareBracket,
    ClosingSquareBracket,
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningCurlyBrace,
    ClosingCurlyBrace,
}

impl OpToken {
    // preview of this token for error messages
    pub fn render_err(&self) -> String {
        match self {
            OpToken::At => "@",
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
            OpToken::BackSlash => "\\",
            OpToken::Dollar => "$",
            OpToken::BackTick => "`",
            OpToken::InclusiveRange => "..~",
            OpToken::Assign => "=",
            OpToken::RArrow => "=>",
            OpToken::Append => "<+",
            OpToken::Prepend => "+>",
            OpToken::OpeningSquareBracket => "[",
            OpToken::ClosingSquareBracket => "]",
            OpToken::OpeningParenthesis => "(",
            OpToken::ClosingParenthesis => ")",
            OpToken::OpeningCurlyBrace => "{",
            OpToken::ClosingCurlyBrace => "}",
        }
        .to_string()
    }
}
