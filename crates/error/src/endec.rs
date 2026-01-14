use crate::{Error, ErrorKind, ErrorToken, NameCollisionKind};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::RenderableSpan;
use sodigy_token::{Delim, Keyword, Punct};

impl Endec for Error {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.kind.encode_impl(buffer);
        self.spans.encode_impl(buffer);
        self.note.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (kind, cursor) = ErrorKind::decode_impl(buffer, cursor)?;
        let (spans, cursor) = Vec::<RenderableSpan>::decode_impl(buffer, cursor)?;
        let (note, cursor) = Option::<String>::decode_impl(buffer, cursor)?;
        Ok((Error { kind, spans, note }, cursor))
    }
}

// `impl Endec for ErrorKind` is implemented in `src/kind.rs` by `error_kinds!()` macro.
// You can find the actual code in `src/proc_macro.rs`.

impl Endec for ErrorToken {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ErrorToken::Nothing => {
                buffer.push(0);
            },
            ErrorToken::Any => {
                buffer.push(1);
            },
            ErrorToken::Character(ch) => {
                buffer.push(2);
                ch.encode_impl(buffer);
            },
            ErrorToken::AnyCharacter => {
                buffer.push(3);
            },
            ErrorToken::Keyword(keyword) => {
                buffer.push(4);
                keyword.encode_impl(buffer);
            },
            ErrorToken::Punct(punct) => {
                buffer.push(5);
                punct.encode_impl(buffer);
            },
            ErrorToken::Group(delim) => {
                buffer.push(6);
                delim.encode_impl(buffer);
            },
            ErrorToken::Ident => {
                buffer.push(7);
            },
            ErrorToken::Generic => {
                buffer.push(8);
            },
            ErrorToken::Number => {
                buffer.push(9);
            },
            ErrorToken::String => {
                buffer.push(10);
            },
            ErrorToken::TypeAnnotation => {
                buffer.push(11);
            },
            ErrorToken::Declaration => {
                buffer.push(12);
            },
            ErrorToken::Expr => {
                buffer.push(13);
            },
            ErrorToken::Path => {
                buffer.push(14);
            },
            ErrorToken::Pattern => {
                buffer.push(15);
            },
            ErrorToken::Item => {
                buffer.push(16);
            },
            ErrorToken::Block => {
                buffer.push(17);
            },
            ErrorToken::Operator => {
                buffer.push(18);
            },
            ErrorToken::AssignOrLt => {
                buffer.push(19);
            },
            ErrorToken::AssignOrSemicolon => {
                buffer.push(20);
            },
            ErrorToken::BraceOrCommaOrParenthesis => {
                buffer.push(21);
            },
            ErrorToken::BraceOrParenthesis => {
                buffer.push(22);
            },
            ErrorToken::ColonOrComma => {
                buffer.push(23);
            },
            ErrorToken::CommaOrDot => {
                buffer.push(24);
            },
            ErrorToken::CommaOrGt => {
                buffer.push(25);
            },
            ErrorToken::DotOrSemicolon => {
                buffer.push(26);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((ErrorToken::Nothing, cursor + 1)),
            Some(1) => Ok((ErrorToken::Any, cursor + 1)),
            Some(2) => {
                let (ch, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Character(ch), cursor))
            },
            Some(3) => Ok((ErrorToken::AnyCharacter, cursor + 1)),
            Some(4) => {
                let (keyword, cursor) = Keyword::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Keyword(keyword), cursor))
            },
            Some(5) => {
                let (punct, cursor) = Punct::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Punct(punct), cursor))
            },
            Some(6) => {
                let (delim, cursor) = Delim::decode_impl(buffer, cursor + 1)?;
                Ok((ErrorToken::Group(delim), cursor))
            },
            Some(7) => Ok((ErrorToken::Ident, cursor + 1)),
            Some(8) => Ok((ErrorToken::Generic, cursor + 1)),
            Some(9) => Ok((ErrorToken::Number, cursor + 1)),
            Some(10) => Ok((ErrorToken::String, cursor + 1)),
            Some(11) => Ok((ErrorToken::TypeAnnotation, cursor + 1)),
            Some(12) => Ok((ErrorToken::Declaration, cursor + 1)),
            Some(13) => Ok((ErrorToken::Expr, cursor + 1)),
            Some(14) => Ok((ErrorToken::Path, cursor + 1)),
            Some(15) => Ok((ErrorToken::Pattern, cursor + 1)),
            Some(16) => Ok((ErrorToken::Item, cursor + 1)),
            Some(17) => Ok((ErrorToken::Block, cursor + 1)),
            Some(18) => Ok((ErrorToken::Operator, cursor + 1)),
            Some(19) => Ok((ErrorToken::AssignOrLt, cursor + 1)),
            Some(20) => Ok((ErrorToken::AssignOrSemicolon, cursor + 1)),
            Some(21) => Ok((ErrorToken::BraceOrCommaOrParenthesis, cursor + 1)),
            Some(22) => Ok((ErrorToken::BraceOrParenthesis, cursor + 1)),
            Some(23) => Ok((ErrorToken::ColonOrComma, cursor + 1)),
            Some(24) => Ok((ErrorToken::CommaOrDot, cursor + 1)),
            Some(25) => Ok((ErrorToken::CommaOrGt, cursor + 1)),
            Some(26) => Ok((ErrorToken::DotOrSemicolon, cursor + 1)),
            Some(n @ 27..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for NameCollisionKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            NameCollisionKind::Block { is_top_level } => {
                buffer.push(0);
                is_top_level.encode_impl(buffer);
            },
            NameCollisionKind::Enum => {
                buffer.push(1);
            },
            NameCollisionKind::Func { params, generics } => {
                buffer.push(2);
                params.encode_impl(buffer);
                generics.encode_impl(buffer);
            },
            NameCollisionKind::Pattern => {
                buffer.push(3);
            },
            NameCollisionKind::Struct => {
                buffer.push(4);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (is_top_level, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                Ok((NameCollisionKind::Block { is_top_level }, cursor))
            },
            Some(1) => Ok((NameCollisionKind::Enum, cursor + 1)),
            Some(2) => {
                let (params, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (generics, cursor) = bool::decode_impl(buffer, cursor)?;

                Ok((NameCollisionKind::Func { params, generics }, cursor))
            },
            Some(3) => Ok((NameCollisionKind::Pattern, cursor + 1)),
            Some(4) => Ok((NameCollisionKind::Struct, cursor + 1)),
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
