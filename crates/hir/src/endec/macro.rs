use crate::{Expr, MacroKind, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_string::InternedString;

impl Endec for MacroKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            MacroKind::IncludeString { path } => {
                buffer.push(0);
                path.encode_impl(buffer);
            },
            MacroKind::IncludeBytes { path } => {
                buffer.push(1);
                path.encode_impl(buffer);
            },
            MacroKind::TypeName { r#type } => {
                buffer.push(2);
                r#type.encode_impl(buffer);
            },
            MacroKind::TypeNameOfValue { value } => {
                buffer.push(3);
                value.encode_impl(buffer);
            },
            MacroKind::NumberOfVariants { r#type } => {
                buffer.push(4);
                r#type.encode_impl(buffer);
            },
            MacroKind::NumberOfFields { r#type } => {
                buffer.push(5);
                r#type.encode_impl(buffer);
            },
            MacroKind::NameOfVariants { r#type } => {
                buffer.push(6);
                r#type.encode_impl(buffer);
            },
            MacroKind::NameOfFields { r#type } => {
                buffer.push(7);
                r#type.encode_impl(buffer);
            },
            MacroKind::File => {
                buffer.push(8);
            },
            MacroKind::ModulePath => {
                buffer.push(9);
            },
            MacroKind::Line => {
                buffer.push(10);
            },
            MacroKind::Column => {
                buffer.push(11);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (path, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::IncludeString { path }, cursor))
            },
            Some(1) => {
                let (path, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::IncludeBytes { path }, cursor))
            },
            Some(2) => {
                let (r#type, cursor) = Type::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::TypeName { r#type }, cursor))
            },
            Some(3) => {
                let (value, cursor) = Expr::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::TypeNameOfValue { value }, cursor))
            },
            Some(4) => {
                let (r#type, cursor) = Type::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::NumberOfVariants { r#type }, cursor))
            },
            Some(5) => {
                let (r#type, cursor) = Type::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::NumberOfFields { r#type }, cursor))
            },
            Some(6) => {
                let (r#type, cursor) = Type::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::NameOfVariants { r#type }, cursor))
            },
            Some(7) => {
                let (r#type, cursor) = Type::decode_impl(buffer, cursor + 1)?;
                Ok((MacroKind::NameOfFields { r#type }, cursor))
            },
            Some(8) => Ok((MacroKind::File, cursor + 1)),
            Some(9) => Ok((MacroKind::ModulePath, cursor + 1)),
            Some(10) => Ok((MacroKind::Line, cursor + 1)),
            Some(11) => Ok((MacroKind::Column, cursor + 1)),
            Some(n @ 12..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
