use crate::Session;
use sodigy_endec::{DecodeError, DumpSession, Endec};
use sodigy_error::{Error, Warning};
use sodigy_mir::Type;
use sodigy_span::Span;
use std::collections::HashMap;

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.types.encode_impl(buffer);
        self.generic_args.encode_impl(buffer);
        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (types, cursor) = HashMap::<Span, Type>::decode_impl(buffer, cursor)?;
        let (generic_args, cursor) = HashMap::<(Span, Span), Type>::decode_impl(buffer, cursor)?;
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                types,
                generic_args,
                errors,
                warnings,
            },
            cursor,
        ))
    }
}

impl DumpSession for Session {
    fn dump_session(&self) -> Vec<u8> {
        let s = format!(
            "{{ types: {:?}, generic_args: {:?} }}",
            self.types,
            self.generic_args,
        );
        let mut c = sodigy_prettify::Context::new(s.as_bytes().to_vec());
        c.step_all();
        c.output().to_vec()
    }
}
