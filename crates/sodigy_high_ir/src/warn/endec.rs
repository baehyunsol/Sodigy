use super::{HirWarning, HirWarningKind};
use crate::names::NameBindingType;
use crate::pattern::{NumberLike, RangeType};
use smallvec::SmallVec;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::{ExtraErrInfo, SodigyError};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for HirWarning {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(HirWarning {
            kind: HirWarningKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for HirWarningKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            HirWarningKind::RedefPrelude(s) => {
                buffer.push(0);
                s.encode(buffer, session);
            },
            HirWarningKind::UnusedName(name, nbt) => {
                buffer.push(1);
                name.encode(buffer, session);
                nbt.encode(buffer, session);
            },
            HirWarningKind::UnnecessaryParen { is_brace } => {
                buffer.push(2);
                is_brace.encode(buffer, session);
            },
            HirWarningKind::PointRange {
                from, to, ty,
            } => {
                buffer.push(3);
                from.encode(buffer, session);
                to.encode(buffer, session);
                ty.encode(buffer, session);
            },
            HirWarningKind::NameBindingOnWildcard => { buffer.push(4); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(HirWarningKind::RedefPrelude(InternedString::decode(buffer, index, session)?)),
                    1 => Ok(HirWarningKind::UnusedName(
                        InternedString::decode(buffer, index, session)?,
                        NameBindingType::decode(buffer, index, session)?,
                    )),
                    2 => Ok(HirWarningKind::UnnecessaryParen {
                        is_brace: bool::decode(buffer, index, session)?,
                    }),
                    3 => Ok(HirWarningKind::PointRange {
                        from: NumberLike::decode(buffer, index, session)?,
                        to: NumberLike::decode(buffer, index, session)?,
                        ty: RangeType::decode(buffer, index, session)?,
                    }),
                    4 => Ok(HirWarningKind::NameBindingOnWildcard),
                    5.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for HirWarning {
    fn dump_json(&self) -> JsonObj {
        self.dump_json_impl()
    }
}
