use super::{HirWarning, HirWarningKind};
use crate::names::NameBindingType;
use crate::pattern::{NumberLike, RangeType};
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::ExtraErrInfo;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for HirWarning {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.spans.encode(buf, session);
        self.extra.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(HirWarning {
            kind: HirWarningKind::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            extra: ExtraErrInfo::decode(buf, index, session)?,
        })
    }
}

impl Endec for HirWarningKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            HirWarningKind::RedefPrelude(s) => {
                buf.push(0);
                s.encode(buf, session);
            },
            HirWarningKind::UnusedName(name, nbt) => {
                buf.push(1);
                name.encode(buf, session);
                nbt.encode(buf, session);
            },
            HirWarningKind::UnnecessaryParen { is_brace } => {
                buf.push(2);
                is_brace.encode(buf, session);
            },
            HirWarningKind::PointRange {
                from, to, ty,
            } => {
                buf.push(3);
                from.encode(buf, session);
                to.encode(buf, session);
                ty.encode(buf, session);
            },
            HirWarningKind::NameBindingOnWildcard => { buf.push(4); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(HirWarningKind::RedefPrelude(InternedString::decode(buf, index, session)?)),
                    1 => Ok(HirWarningKind::UnusedName(
                        InternedString::decode(buf, index, session)?,
                        NameBindingType::decode(buf, index, session)?,
                    )),
                    2 => Ok(HirWarningKind::UnnecessaryParen {
                        is_brace: bool::decode(buf, index, session)?,
                    }),
                    3 => Ok(HirWarningKind::PointRange {
                        from: NumberLike::decode(buf, index, session)?,
                        to: NumberLike::decode(buf, index, session)?,
                        ty: RangeType::decode(buf, index, session)?,
                    }),
                    4 => Ok(HirWarningKind::NameBindingOnWildcard),
                    5.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
