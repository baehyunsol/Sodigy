use crate::{
    Delim,
    FormattedStringElement,
    Punct,
    QuoteKind,
    TokenTree,
    TokenTreeKind,
};
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_keyword::Keyword;
use sodigy_span::SpanRange;

impl Endec for TokenTree {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.span.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(TokenTree {
            kind: TokenTreeKind::decode(buf, index, session)?,
            span: SpanRange::decode(buf, index, session)?,
        })
    }
}

impl Endec for TokenTreeKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            TokenTreeKind::Identifier(id) => {
                buf.push(0);
                id.encode(buf, session);
            },
            TokenTreeKind::Keyword(k) => {
                buf.push(1);
                k.encode(buf, session);
            },
            TokenTreeKind::Number(n) => {
                buf.push(2);
                n.encode(buf, session);
            },
            TokenTreeKind::Punct(p) => {
                buf.push(3);
                p.encode(buf, session);
            },
            TokenTreeKind::Group { delim, tokens, prefix } => {
                buf.push(4);
                delim.encode(buf, session);
                tokens.encode(buf, session);
                prefix.encode(buf, session);
            },
            TokenTreeKind::String { kind, content, is_binary } => {
                buf.push(5);
                kind.encode(buf, session);
                content.encode(buf, session);
                is_binary.encode(buf, session);
            },
            TokenTreeKind::FormattedString(e) => {
                buf.push(6);
                e.encode(buf, session);
            },
            TokenTreeKind::DocComment(c) => {
                buf.push(7);
                c.encode(buf, session);
            },
            TokenTreeKind::Macro { name, args } => {
                buf.push(8);
                name.encode(buf, session);
                args.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(TokenTreeKind::Identifier(InternedString::decode(buf, index, session)?)),
                    1 => Ok(TokenTreeKind::Keyword(Keyword::decode(buf, index, session)?)),
                    2 => Ok(TokenTreeKind::Number(InternedNumeric::decode(buf, index, session)?)),
                    3 => Ok(TokenTreeKind::Punct(Punct::decode(buf, index, session)?)),
                    4 => Ok(TokenTreeKind::Group {
                        delim: Delim::decode(buf, index, session)?,
                        tokens: Vec::<TokenTree>::decode(buf, index, session)?,
                        prefix: u8::decode(buf, index, session)?,
                    }),
                    5 => Ok(TokenTreeKind::String {
                        kind: QuoteKind::decode(buf, index, session)?,
                        content: InternedString::decode(buf, index, session)?,
                        is_binary: bool::decode(buf, index, session)?,
                    }),
                    6 => Ok(TokenTreeKind::FormattedString(Vec::<FormattedStringElement>::decode(buf, index, session)?)),
                    7 => Ok(TokenTreeKind::DocComment(InternedString::decode(buf, index, session)?)),
                    8 => Ok(TokenTreeKind::Macro {
                        name: Vec::<TokenTree>::decode(buf, index, session)?,
                        args: Vec::<TokenTree>::decode(buf, index, session)?,
                    }),
                    9.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

#[cfg(test)]
use sodigy_intern::{intern_numeric, intern_string};

#[cfg(test)]
use sodigy_number::SodigyNumber;

#[test]
fn endec_test() {
    let sample = vec![
        TokenTree {
            kind: TokenTreeKind::DocComment(intern_string((b"This is a doc-comment.").to_vec())),
            span: SpanRange::dummy(10),
        },
        TokenTree {
            kind: TokenTreeKind::Punct(Punct::At),
            span: SpanRange::dummy(100),
        },
        TokenTree {
            kind: TokenTreeKind::Identifier(intern_string((b"test").to_vec())),
            span: SpanRange::dummy(101),
        },
        TokenTree {
            kind: TokenTreeKind::Punct(Punct::Dot),
            span: SpanRange::dummy(102),
        },
        TokenTree {
            kind: TokenTreeKind::Identifier(intern_string((b"eq").to_vec())),
            span: SpanRange::dummy(103),
        },
        TokenTree {
            kind: TokenTreeKind::Group {
                delim: Delim::Paren,
                prefix: b'\0',
                tokens: vec![
                    TokenTree {
                        kind: TokenTreeKind::Number(intern_numeric(SodigyNumber::SmallInt(0))),
                        span: SpanRange::dummy(104),
                    }
                ],
            },
            span: SpanRange::dummy(105),
        },
    ];
    let mut buf = vec![];
    let mut session = EndecSession::new();

    sample.encode(&mut buf, &mut session);

    let mut index = 0;
    let reconstructed = Vec::<TokenTree>::decode(&buf, &mut index, &mut session).unwrap();

    assert_eq!(
        format!("{sample:?}"),
        format!("{reconstructed:?}"),
    );
}
