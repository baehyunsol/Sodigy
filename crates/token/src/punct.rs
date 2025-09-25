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
    Shl,  // "<<"
    Shr,  // ">>"
    Eq,   // "=="
    Leq,  // "<="
    Neq,  // "!="
    Geq,  // ">="
    DotDot,  // ".."
    Arrow,  // "=>"
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
            _ => panic!("TODO: {:?}", b as char),
        }
    }
}
