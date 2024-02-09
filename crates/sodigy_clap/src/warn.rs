use crate::stages::IrStage;
use crate::flag::Flag;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    ErrorContext,
    ExtraErrInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct ClapWarning {
    kind: ClapWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl ClapWarning {
    pub fn ext_mismatch(ext: IrStage, format: IrStage) -> Self {
        ClapWarning {
            kind: ClapWarningKind::ExtMismatch { ext, format },
            spans: smallvec![],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn same_flag_multiple_times(flag: Flag, span: SpanRange) -> Self {
        ClapWarning {
            kind: ClapWarningKind::SameFlagMultipleTimes(flag),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn path_is_set_flag_is_not_set(
        is_set: Flag,
        is_not_set: Flag,
        spans: Vec<SpanRange>,
    ) -> Self {
        ClapWarning {
            kind: ClapWarningKind::PathIsSetFlagIsNotSet { is_set, is_not_set },
            spans: spans.into(),
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }
}

impl SodigyError<ClapWarningKind> for ClapWarning {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &ClapWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    // do we even need this?
    fn index(&self) -> u32 {
        3
    }
}

pub enum ClapWarningKind {
    ExtMismatch {
        ext: IrStage,
        format: IrStage,
    },
    SameFlagMultipleTimes(Flag),
    PathIsSetFlagIsNotSet {
        is_set: Flag,
        is_not_set: Flag,
    },
}

impl SodigyErrorKind for ClapWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ClapWarningKind::ExtMismatch { .. } => String::from("mismatch between the extension of path and `--to` option"),
            ClapWarningKind::SameFlagMultipleTimes(flag) => format!("`{}` given more than once", flag.render_error()),
            ClapWarningKind::PathIsSetFlagIsNotSet { is_set, is_not_set } => format!("`{}` is set, but `{}` is not set", is_set.render_error(), is_not_set.render_error()),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ClapWarningKind::ExtMismatch { ext, format } => format!(
                "The extension is `{}`, but the given format is `{}`.",
                ext.render_error(),
                format.render_error(),
            ),
            ClapWarningKind::SameFlagMultipleTimes(_) => String::new(),
            ClapWarningKind::PathIsSetFlagIsNotSet { is_set, .. } => format!("`{}` doesn't do anything.", is_set.render_error()),
        }
    }

    // we don't need this, but I want it to look more complete
    fn index(&self) -> u32 {
        match self {
            ClapWarningKind::ExtMismatch { .. } => 0,
            ClapWarningKind::SameFlagMultipleTimes(_) => 1,
            ClapWarningKind::PathIsSetFlagIsNotSet { .. } => 2,
        }
    }
}
