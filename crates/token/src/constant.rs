use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

// TODO: we can derive `Copy` if `InternedNumber` does so!
#[derive(Clone, Debug)]
pub enum Constant {
    Number {
        n: InternedNumber,
        span: Span,
    },
    String {
        binary: bool,
        s: InternedString,

        // it includes quotes
        span: Span,
    },
    Char {
        ch: u32,

        // it includes quotes
        span: Span,
    },
    Byte {
        b: u8,
        span: Span,
    },
}

impl Constant {
    pub fn span(&self) -> Span {
        match self {
            Constant::Number { span, .. } |
            Constant::String { span, .. } |
            Constant::Char { span, .. } |
            Constant::Byte { span, .. } => *span,
        }
    }

    /// If you see this value in bytecode, it's 99% likely that there's a bug in the compiler.
    pub fn dummy() -> Self {
        Constant::Char {
            ch: '쉵' as u32,
            span: Span::None,
        }
    }

    pub fn dump(&self, intermediate_dir: &str) -> String {
        match self {
            Constant::Number { n, .. } => n.dump(),
            Constant::String { binary, s, .. } => format!(
                "{}{:?}",
                if *binary { "b" } else { "" },
                s.unintern_or_default(intermediate_dir),
            ),
            Constant::Char { ch, .. } => format!("{:?}", char::from_u32(*ch).unwrap()),
            Constant::Byte { b, .. } => format!("#{b}"),
        }
    }
}
