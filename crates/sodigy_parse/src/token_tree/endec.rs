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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(TokenTree {
            kind: TokenTreeKind::decode(buf, ind, session)?,
            span: SpanRange::decode(buf, ind, session)?,
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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(TokenTreeKind::Identifier(InternedString::decode(buf, ind, session)?)),
                    1 => Ok(TokenTreeKind::Keyword(Keyword::decode(buf, ind, session)?)),
                    2 => Ok(TokenTreeKind::Number(InternedNumeric::decode(buf, ind, session)?)),
                    3 => Ok(TokenTreeKind::Punct(Punct::decode(buf, ind, session)?)),
                    4 => Ok(TokenTreeKind::Group {
                        delim: Delim::decode(buf, ind, session)?,
                        tokens: Vec::<TokenTree>::decode(buf, ind, session)?,
                        prefix: u8::decode(buf, ind, session)?,
                    }),
                    5 => Ok(TokenTreeKind::String {
                        kind: QuoteKind::decode(buf, ind, session)?,
                        content: InternedString::decode(buf, ind, session)?,
                        is_binary: bool::decode(buf, ind, session)?,
                    }),
                    6 => Ok(TokenTreeKind::FormattedString(Vec::<FormattedStringElement>::decode(buf, ind, session)?)),
                    7 => Ok(TokenTreeKind::DocComment(InternedString::decode(buf, ind, session)?)),
                    8 => Ok(TokenTreeKind::Macro {
                        name: Vec::<TokenTree>::decode(buf, ind, session)?,
                        args: Vec::<TokenTree>::decode(buf, ind, session)?,
                    }),
                    9.. => Err(EndecError::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecError::Eof),
        }
    }
}
