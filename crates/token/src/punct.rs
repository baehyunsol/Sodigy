#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Punct {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Colon,
    Semicolon,
    Assign,
    Lt,
    Gt,
    Comma,
    Dot,
    QuestionMark,
    Factorial,
    At,
    Dollar,
    And,  // "&"
    Or,   // "|"
    Xor,  // "^"
    AndAnd,  // "&&"
    OrOr,    // "||"
    Shl,  // "<<"
    Shr,  // ">>"
    Eq,   // "=="
    Leq,  // "<="
    Neq,  // "!="
    Geq,  // ">="
    Concat,  // "++"
    Append,  // "<+"
    Prepend, // "+>"
    DotDot,  // ".."
    DotDotEq,  // "..="
    Arrow,  // "=>"
    ReturnType,  // "->"
    Pipeline,  // "|>"
}

impl Punct {
    // Used when generating error messages.
    pub fn render_error(&self) -> &'static str {
        match self {
            Punct::Add => "+",
            Punct::Sub => "-",
            Punct::Mul => "*",
            Punct::Div => "/",
            Punct::Rem => "%",
            Punct::Colon => ":",
            Punct::Semicolon => ";",
            Punct::Assign => "=",
            Punct::Lt => "<",
            Punct::Gt => ">",
            Punct::Comma => ",",
            Punct::Dot => ".",
            Punct::QuestionMark => "?",
            Punct::Factorial => "!",
            Punct::And => "&&",
            _ => panic!("TODO: {self:?}"),
        }
    }
}

impl From<u8> for Punct {
    fn from(b: u8) -> Punct {
        match b {
            b'!' => Punct::Factorial,
            b'$' => Punct::Dollar,
            b'%' => Punct::Rem,
            b'&' => Punct::And,
            b'*' => Punct::Mul,
            b'+' => Punct::Add,
            b',' => Punct::Comma,
            b'-' => Punct::Sub,
            b'.' => Punct::Dot,
            b'/' => Punct::Div,
            b':' => Punct::Colon,
            b';' => Punct::Semicolon,
            b'<' => Punct::Lt,
            b'=' => Punct::Assign,
            b'>' => Punct::Gt,
            b'?' => Punct::QuestionMark,
            b'@' => Punct::At,
            b'^' => Punct::Xor,
            b'|' => Punct::Or,
            _ => panic!("TODO: {:?}", b as char),
        }
    }
}
