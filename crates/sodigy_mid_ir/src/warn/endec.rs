use super::{MirWarning, MirWarningKind};
use smallvec::SmallVec;
use sodigy_endec::{
    Endec,
    EndecError,
    EndecSession,
};
use sodigy_error::ExtraErrorInfo;
use sodigy_high_ir::NameBindingType;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for MirWarning {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(MirWarning {
            kind: MirWarningKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrorInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for MirWarningKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            MirWarningKind::UnusedLocalValue { name, name_binding_type, no_ref_at_all } => {
                buffer.push(0);
                name.encode(buffer, session);
                name_binding_type.encode(buffer, session);
                no_ref_at_all.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(MirWarningKind::UnusedLocalValue {
                        name: InternedString::decode(buffer, index, session)?,
                        name_binding_type: NameBindingType::decode(buffer, index, session)?,
                        no_ref_at_all: bool::decode(buffer, index, session)?,
                    }),
                    1.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
