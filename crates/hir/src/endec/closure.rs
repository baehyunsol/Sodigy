use crate::CapturedNames;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for CapturedNames {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.locals.encode_impl(buffer);
        self.globals.encode_impl(buffer);
        self.constants.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (locals, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;
        let (globals, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;
        let (constants, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            CapturedNames {
                locals,
                globals,
                constants,
            },
            cursor,
        ))
    }
}
