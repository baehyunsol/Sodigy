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
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.span.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(TokenTree {
            kind: TokenTreeKind::decode(buffer, index, session)?,
            span: SpanRange::decode(buffer, index, session)?,
        })
    }
}

impl Endec for TokenTreeKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            TokenTreeKind::Identifier(id) => {
                buffer.push(0);
                id.encode(buffer, session);
            },
            TokenTreeKind::Keyword(k) => {
                buffer.push(1);
                k.encode(buffer, session);
            },
            TokenTreeKind::Number(n) => {
                buffer.push(2);
                n.encode(buffer, session);
            },
            TokenTreeKind::Punct(p) => {
                buffer.push(3);
                p.encode(buffer, session);
            },
            TokenTreeKind::Group { delim, tokens, prefix } => {
                buffer.push(4);
                delim.encode(buffer, session);
                tokens.encode(buffer, session);
                prefix.encode(buffer, session);
            },
            TokenTreeKind::String { kind, content, is_binary } => {
                buffer.push(5);
                kind.encode(buffer, session);
                content.encode(buffer, session);
                is_binary.encode(buffer, session);
            },
            TokenTreeKind::FormattedString(e) => {
                buffer.push(6);
                e.encode(buffer, session);
            },
            TokenTreeKind::DocComment(c) => {
                buffer.push(7);
                c.encode(buffer, session);
            },
            TokenTreeKind::Macro { name, args } => {
                buffer.push(8);
                name.encode(buffer, session);
                args.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(TokenTreeKind::Identifier(InternedString::decode(buffer, index, session)?)),
                    1 => Ok(TokenTreeKind::Keyword(Keyword::decode(buffer, index, session)?)),
                    2 => Ok(TokenTreeKind::Number(InternedNumeric::decode(buffer, index, session)?)),
                    3 => Ok(TokenTreeKind::Punct(Punct::decode(buffer, index, session)?)),
                    4 => Ok(TokenTreeKind::Group {
                        delim: Delim::decode(buffer, index, session)?,
                        tokens: Vec::<TokenTree>::decode(buffer, index, session)?,
                        prefix: u8::decode(buffer, index, session)?,
                    }),
                    5 => Ok(TokenTreeKind::String {
                        kind: QuoteKind::decode(buffer, index, session)?,
                        content: InternedString::decode(buffer, index, session)?,
                        is_binary: bool::decode(buffer, index, session)?,
                    }),
                    6 => Ok(TokenTreeKind::FormattedString(Vec::<FormattedStringElement>::decode(buffer, index, session)?)),
                    7 => Ok(TokenTreeKind::DocComment(InternedString::decode(buffer, index, session)?)),
                    8 => Ok(TokenTreeKind::Macro {
                        name: InternedString::decode(buffer, index, session)?,
                        args: Vec::<TokenTree>::decode(buffer, index, session)?,
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
            span: SpanRange::dummy(),
        },
        TokenTree {
            kind: TokenTreeKind::Punct(Punct::At),
            span: SpanRange::dummy(),
        },
        TokenTree {
            kind: TokenTreeKind::Identifier(intern_string((b"test").to_vec())),
            span: SpanRange::dummy(),
        },
        TokenTree {
            kind: TokenTreeKind::Punct(Punct::Dot),
            span: SpanRange::dummy(),
        },
        TokenTree {
            kind: TokenTreeKind::Identifier(intern_string((b"eq").to_vec())),
            span: SpanRange::dummy(),
        },
        TokenTree {
            kind: TokenTreeKind::Group {
                delim: Delim::Paren,
                prefix: b'\0',
                tokens: vec![
                    TokenTree {
                        kind: TokenTreeKind::Number(intern_numeric(SodigyNumber::SmallInt(0))),
                        span: SpanRange::dummy(),
                    }
                ],
            },
            span: SpanRange::dummy(),
        },
    ];
    let mut buffer = vec![];
    let mut session = EndecSession::new();

    sample.encode(&mut buffer, &mut session);

    let mut index = 0;
    let reconstructed = Vec::<TokenTree>::decode(&buffer, &mut index, &mut session).unwrap();

    assert_eq!(
        format!("{sample:?}"),
        format!("{reconstructed:?}"),
    );
}
