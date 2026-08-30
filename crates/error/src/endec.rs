use crate::{
    EnumFieldKind,
    Error,
    ErrorKind,
    ErrorToken,
    FuncEffect,
    ItemKind,
    NameCollisionKind,
    NotXBut,
    ParamIndex,
    TypeVarInfo,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
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
            ErrorToken::Wildcard => {
                buffer.push(7);
            },
            ErrorToken::Ident => {
                buffer.push(8);
            },
            ErrorToken::Generic => {
                buffer.push(9);
            },
            ErrorToken::Number => {
                buffer.push(10);
            },
            ErrorToken::String => {
                buffer.push(11);
            },
            ErrorToken::FieldUpdate => {
                buffer.push(12);
            },
            ErrorToken::DocComment => {
                buffer.push(13);
            },
            ErrorToken::TypeAnnot => {
                buffer.push(14);
            },
            ErrorToken::Declaration => {
                buffer.push(15);
            },
            ErrorToken::Expr => {
                buffer.push(16);
            },
            ErrorToken::Path => {
                buffer.push(17);
            },
            ErrorToken::Pattern => {
                buffer.push(18);
            },
            ErrorToken::Item => {
                buffer.push(19);
            },
            ErrorToken::Block => {
                buffer.push(20);
            },
            ErrorToken::Operator => {
                buffer.push(21);
            },
            ErrorToken::LambdaParams => {
                buffer.push(22);
            },
            ErrorToken::AssignOrColon => {
                buffer.push(23);
            },
            ErrorToken::AssignOrLt => {
                buffer.push(24);
            },
            ErrorToken::AssignOrSemicolon => {
                buffer.push(25);
            },
            ErrorToken::BraceOrCommaOrParenthesis => {
                buffer.push(26);
            },
            ErrorToken::BraceOrParenthesis => {
                buffer.push(27);
            },
            ErrorToken::ColonOrComma => {
                buffer.push(28);
            },
            ErrorToken::CommaOrDot => {
                buffer.push(29);
            },
            ErrorToken::CommaOrGt => {
                buffer.push(30);
            },
            ErrorToken::DotOrSemicolon => {
                buffer.push(31);
            },
            ErrorToken::FnOrNdetOrProc => {
                buffer.push(32);
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
            Some(7) => Ok((ErrorToken::Wildcard, cursor + 1)),
            Some(8) => Ok((ErrorToken::Ident, cursor + 1)),
            Some(9) => Ok((ErrorToken::Generic, cursor + 1)),
            Some(10) => Ok((ErrorToken::Number, cursor + 1)),
            Some(11) => Ok((ErrorToken::String, cursor + 1)),
            Some(12) => Ok((ErrorToken::FieldUpdate, cursor + 1)),
            Some(13) => Ok((ErrorToken::DocComment, cursor + 1)),
            Some(14) => Ok((ErrorToken::TypeAnnot, cursor + 1)),
            Some(15) => Ok((ErrorToken::Declaration, cursor + 1)),
            Some(16) => Ok((ErrorToken::Expr, cursor + 1)),
            Some(17) => Ok((ErrorToken::Path, cursor + 1)),
            Some(18) => Ok((ErrorToken::Pattern, cursor + 1)),
            Some(19) => Ok((ErrorToken::Item, cursor + 1)),
            Some(20) => Ok((ErrorToken::Block, cursor + 1)),
            Some(21) => Ok((ErrorToken::Operator, cursor + 1)),
            Some(22) => Ok((ErrorToken::LambdaParams, cursor + 1)),
            Some(23) => Ok((ErrorToken::AssignOrColon, cursor + 1)),
            Some(24) => Ok((ErrorToken::AssignOrLt, cursor + 1)),
            Some(25) => Ok((ErrorToken::AssignOrSemicolon, cursor + 1)),
            Some(26) => Ok((ErrorToken::BraceOrCommaOrParenthesis, cursor + 1)),
            Some(27) => Ok((ErrorToken::BraceOrParenthesis, cursor + 1)),
            Some(28) => Ok((ErrorToken::ColonOrComma, cursor + 1)),
            Some(29) => Ok((ErrorToken::CommaOrDot, cursor + 1)),
            Some(30) => Ok((ErrorToken::CommaOrGt, cursor + 1)),
            Some(31) => Ok((ErrorToken::DotOrSemicolon, cursor + 1)),
            Some(32) => Ok((ErrorToken::FnOrNdetOrProc, cursor + 1)),
            Some(n @ 33..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
            NameCollisionKind::EnumGeneric => {
                buffer.push(2);
            },
            NameCollisionKind::Func { params, generics } => {
                buffer.push(3);
                params.encode_impl(buffer);
                generics.encode_impl(buffer);
            },
            NameCollisionKind::Pattern => {
                buffer.push(4);
            },
            NameCollisionKind::Struct => {
                buffer.push(5);
            },
            NameCollisionKind::StructGeneric => {
                buffer.push(6);
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
            Some(2) => Ok((NameCollisionKind::EnumGeneric, cursor + 1)),
            Some(3) => {
                let (params, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (generics, cursor) = bool::decode_impl(buffer, cursor)?;

                Ok((NameCollisionKind::Func { params, generics }, cursor))
            },
            Some(4) => Ok((NameCollisionKind::Pattern, cursor + 1)),
            Some(5) => Ok((NameCollisionKind::Struct, cursor + 1)),
            Some(6) => Ok((NameCollisionKind::StructGeneric, cursor + 1)),
            Some(n @ 7..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for NotXBut {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            NotXBut::Expr => {
                buffer.push(0);
            },
            NotXBut::Struct => {
                buffer.push(1);
            },
            NotXBut::Enum => {
                buffer.push(2);
            },
            NotXBut::Module => {
                buffer.push(3);
            },
            NotXBut::GenericParam => {
                buffer.push(4);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((NotXBut::Expr, cursor + 1)),
            Some(1) => Ok((NotXBut::Struct, cursor + 1)),
            Some(2) => Ok((NotXBut::Enum, cursor + 1)),
            Some(3) => Ok((NotXBut::Module, cursor + 1)),
            Some(4) => Ok((NotXBut::GenericParam, cursor + 1)),
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ParamIndex {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ParamIndex::Param(n) => {
                buffer.push(0);
                n.encode_impl(buffer);
            },
            ParamIndex::Return => {
                buffer.push(1);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((ParamIndex::Param(n), cursor))
            },
            Some(1) => Ok((ParamIndex::Return, cursor + 1)),
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for EnumFieldKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            EnumFieldKind::None => {
                buffer.push(0);
            },
            EnumFieldKind::Tuple => {
                buffer.push(1);
            },
            EnumFieldKind::Struct => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((EnumFieldKind::None, cursor + 1)),
            Some(1) => Ok((EnumFieldKind::Tuple, cursor + 1)),
            Some(2) => Ok((EnumFieldKind::Struct, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for TypeVarInfo {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            TypeVarInfo::Ident(id) => {
                buffer.push(0);
                id.encode_impl(buffer);
            },
            TypeVarInfo::ListExpr => {
                buffer.push(1);
            },
            TypeVarInfo::ListPattern => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((TypeVarInfo::Ident(id), cursor))
            },
            Some(1) => Ok((TypeVarInfo::ListExpr, cursor + 1)),
            Some(2) => Ok((TypeVarInfo::ListPattern, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ItemKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ItemKind::Alias => {
                buffer.push(0);
            },
            ItemKind::Assert => {
                buffer.push(1);
            },
            ItemKind::Enum => {
                buffer.push(2);
            },
            ItemKind::EnumVariant => {
                buffer.push(3);
            },
            ItemKind::Func => {
                buffer.push(4);
            },
            ItemKind::FuncParam => {
                buffer.push(5);
            },
            ItemKind::StructField => {
                buffer.push(6);
            },
            ItemKind::Let => {
                buffer.push(7);
            },
            ItemKind::Module => {
                buffer.push(8);
            },
            ItemKind::Struct => {
                buffer.push(9);
            },
            ItemKind::Use => {
                buffer.push(10);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((ItemKind::Alias, cursor + 1)),
            Some(1) => Ok((ItemKind::Assert, cursor + 1)),
            Some(2) => Ok((ItemKind::Enum, cursor + 1)),
            Some(3) => Ok((ItemKind::EnumVariant, cursor + 1)),
            Some(4) => Ok((ItemKind::Func, cursor + 1)),
            Some(5) => Ok((ItemKind::FuncParam, cursor + 1)),
            Some(6) => Ok((ItemKind::StructField, cursor + 1)),
            Some(7) => Ok((ItemKind::Let, cursor + 1)),
            Some(8) => Ok((ItemKind::Module, cursor + 1)),
            Some(9) => Ok((ItemKind::Struct, cursor + 1)),
            Some(10) => Ok((ItemKind::Use, cursor + 1)),
            Some(n @ 11..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for FuncEffect {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            FuncEffect::Fn => {
                buffer.push(0);
            },
            FuncEffect::Proc => {
                buffer.push(1);
            },
            FuncEffect::NdetFn => {
                buffer.push(2);
            },
            FuncEffect::NdetProc => {
                buffer.push(3);
            },
            FuncEffect::Callable => {
                buffer.push(4);
            },
            FuncEffect::Var(s) => {
                buffer.push(5);
                s.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((FuncEffect::Fn, cursor + 1)),
            Some(1) => Ok((FuncEffect::Proc, cursor + 1)),
            Some(2) => Ok((FuncEffect::NdetFn, cursor + 1)),
            Some(3) => Ok((FuncEffect::NdetProc, cursor + 1)),
            Some(4) => Ok((FuncEffect::Callable, cursor + 1)),
            Some(5) => {
                let (s, cursor) = Box::<Span>::decode_impl(buffer, cursor + 1)?;
                Ok((FuncEffect::Var(s), cursor))
            },
            Some(n @ 6..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
