use crate::DumpType;
use crate::arg::Arg;
use crate::error::ClapError;
use crate::lex::Token;
use hmath::BigInt;

mod fmt;

#[derive(Clone, PartialEq)]
pub enum TokenKind {
    None,
    Path,

    // (NAME=PATH)+
    Library,
    String,
    Integer,

    Optional(Box<TokenKind>),
    Flag,
    DumpType,
    EqualSign,  // for error messages
}

impl TokenKind {
    pub fn try_parse_arg(&self, token: &Token) -> Result<Arg, ClapError> {
        match self {
            TokenKind::None
            | TokenKind::Library
            | TokenKind::Flag
            | TokenKind::EqualSign
            | TokenKind::Optional(_) => unreachable!(),
            // it doesn't allow path starting with "-" or "=", in order to prevent confusion
            // if the user really wants to use such paths, the path have to be prefixed with "./"
            TokenKind::Path => if token.buffer.get(0) == Some(&b'-') || token.buffer == b"=" {
                Err(ClapError::invalid_argument(
                    TokenKind::Path,
                    &token.buffer,
                    token.span,
                ))
            } else {
                Ok(Arg::Path(
                    String::from_utf8(token.buffer.to_vec()).map_err(
                        |_| ClapError::invalid_utf8(token.span)
                    )?
                ))
            },
            TokenKind::String => Ok(Arg::String(
                String::from_utf8(token.buffer.to_vec()).map_err(
                    |_| ClapError::invalid_utf8(token.span)
                )?
            )),
            TokenKind::Integer => match BigInt::from_string(
                &String::from_utf8(token.buffer.to_vec()).map_err(
                    |_| ClapError::invalid_utf8(token.span)
                )?
            ) {
                Ok(n) => match i64::try_from(n.clone()) {
                    Ok(n) => Ok(Arg::Integer(n)),
                    Err(_) => Err(ClapError::integer_range_error(
                        Some(BigInt::from(i64::MIN)),
                        Some(BigInt::from(i64::MAX).add_i32(1)),
                        n,
                        token.span,
                    )),
                },
                Err(_) => Err(ClapError::invalid_argument(
                    TokenKind::Integer,
                    &token.buffer,
                    token.span,
                )),
            },
            TokenKind::DumpType => match &token.buffer {
                b if b == b"json" => Ok(Arg::DumpType(DumpType::Json)),
                b if b == b"string" => Ok(Arg::DumpType(DumpType::String)),
                _ => Err(ClapError::invalid_argument(
                    TokenKind::DumpType,
                    &token.buffer,
                    token.span,
                )),
            },
        }
    }
}
