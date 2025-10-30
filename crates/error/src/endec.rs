use crate::{Error, ErrorKind};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::RenderableSpan;

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

// TODO: is it okay to use only 1 byte for variant index? What if there are more than 256 variants?
impl Endec for ErrorKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            _ => panic!("TODO: {self:?}"),
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(_) => todo!(),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
