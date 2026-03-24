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

    // The compiler internally converts Byte/Char to Scalar.
    // The users cannot use this value in their Sodigy code.
    // That's why it has no span.
    Scalar(u32),
}

impl Constant {
    pub fn span(&self) -> Span {
        match self {
            Constant::Number { span, .. } |
            Constant::String { span, .. } |
            Constant::Char { span, .. } |
            Constant::Byte { span, .. } => *span,
            Constant::Scalar(_) => Span::None,
        }
    }

    /// If you see this value in bytecode, it's 99% likely that there's a bug in the compiler.
    pub fn dummy() -> Self {
        Constant::Scalar(703459145)
    }

    pub fn dump(&self, intermediate_dir: &str) -> String {
        match self {
            Constant::Number { n, .. } => n.dump(intermediate_dir),
            Constant::String { binary, s, .. } => format!(
                "{}{:?}",
                if *binary { "b" } else { "" },
                s.unintern_or_default(intermediate_dir),
            ),
            Constant::Char { ch, .. } => format!("{:?}", char::from_u32(*ch).unwrap()),
            Constant::Byte { b, .. } => format!("#{b}"),
            Constant::Scalar(n) => format!("__SCALAR__({n})"),
        }
    }

    #[must_use = "method returns a new constant and does not mutate the original constant"]
    pub fn monomorphize(&self, monomorphize_id: u128) -> Self {
        match self {
            Constant::Number { n, span } => Constant::Number {
                n: *n,
                span: span.monomorphize(monomorphize_id),
            },
            Constant::String { binary, s, span } => Constant::String {
                binary: *binary,
                s: *s,
                span: span.monomorphize(monomorphize_id),
            },
            Constant::Char { ch, span } => Constant::Char {
                ch: *ch,
                span: span.monomorphize(monomorphize_id),
            },
            Constant::Byte { b, span } => Constant::Byte {
                b: *b,
                span: span.monomorphize(monomorphize_id),
            },
            Constant::Scalar(n) => Constant::Scalar(*n),
        }
    }
}
