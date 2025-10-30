use crate::Span;
use sodigy_endec::{DecodeError, Endec};
use sodigy_file::File;
use sodigy_string::InternedString;

impl Endec for Span {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Span::File(file) => {
                buffer.push(0);
                file.encode_impl(buffer);
            },
            Span::Range { file, start, end } => {
                buffer.push(1);
                file.encode_impl(buffer);
                start.encode_impl(buffer);
                end.encode_impl(buffer);
            },
            Span::Eof(file) => {
                buffer.push(2);
                file.encode_impl(buffer);
            },
            Span::Prelude(s) => {
                buffer.push(3);
                s.encode_impl(buffer);
            },
            Span::None => {
                buffer.push(4);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                Ok((Span::File(file), cursor))
            },
            Some(1) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                let (start, cursor) = usize::decode_impl(buffer, cursor)?;
                let (end, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((Span::Range { file, start, end }, cursor))
            },
            Some(2) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Eof(file), cursor))
            },
            Some(3) => {
                let (file, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Prelude(file), cursor))
            },
            Some(4) => Ok((Span::None, cursor + 1)),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
