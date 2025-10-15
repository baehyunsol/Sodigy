#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    At,
    Dollar,
    And,  // "&"
    Or,   // "|"
    AndAnd,  // "&&"
    OrOr,    // "||"
    Shl,  // "<<"
    Shr,  // ">>"
    Eq,   // "=="
    Leq,  // "<="
    Neq,  // "!="
    Geq,  // ">="
    Concat,  // "++"
    DotDot,  // ".."
    DotDotEq,  // "..="
    Arrow,  // "=>"
    ReturnType,  // "->"
}

impl From<u8> for Punct {
    fn from(b: u8) -> Punct {
        match b {
            b'%' => Punct::Rem,
            b'*' => Punct::Mul,
            b'+' => Punct::Add,
            b',' => Punct::Comma,
            b'-' => Punct::Sub,
            b'/' => Punct::Div,
            b':' => Punct::Colon,
            b';' => Punct::Semicolon,
            b'=' => Punct::Assign,
            b'<' => Punct::Lt,
            b'>' => Punct::Gt,
            b'.' => Punct::Dot,
            b'?' => Punct::QuestionMark,
            b'@' => Punct::At,
            b'$' => Punct::Dollar,
            b'&' => Punct::And,
            b'|' => Punct::Or,
            _ => panic!("TODO: {:?}", b as char),
        }
    }
}
