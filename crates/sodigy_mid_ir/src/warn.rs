use crate::func::LocalValue;
use smallvec::{SmallVec, smallvec};
use sodigy_error::{
    ExtraErrorInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_high_ir::NameBindingType;
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

mod endec;

pub struct MirWarning {
    kind: MirWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl MirWarning {
    pub fn unused_local_value(local_value: &LocalValue, no_ref_at_all: bool) -> Self {
        MirWarning {
            kind: MirWarningKind::UnusedLocalValue {
                name: local_value.name.id(),
                name_binding_type: local_value.name_binding_type,
                no_ref_at_all,
            },
            spans: smallvec![*local_value.name.span()],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<MirWarningKind> for MirWarning {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrorInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrorInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &MirWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        9
    }

    fn get_stage(&self) -> Stage {
        Stage::Mir
    }
}

pub enum MirWarningKind {
    UnusedLocalValue {
        name: InternedString,
        name_binding_type: NameBindingType,

        // no ref at all
        // vs
        // has ref, but unreachable from the return value
        no_ref_at_all: bool,
    },
}

impl SodigyErrorKind for MirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            MirWarningKind::UnusedLocalValue { name, name_binding_type, .. } => format!(
                "unused {} `{name}`",
                name_binding_type.render_error(),
            ),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            MirWarningKind::UnusedLocalValue { name, no_ref_at_all, .. } => if *no_ref_at_all {
                String::new()
            } else {
                format!(
                    "`{name}` is used by another value, but it's not reachable from the return value."
                )
            },
        }
    }

    fn index(&self) -> u32 {
        match self {
            MirWarningKind::UnusedLocalValue { .. } => 0,
        }
    }
}
