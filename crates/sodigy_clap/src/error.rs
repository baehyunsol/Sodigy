use crate::flag::FLAGS;
use crate::token::TokenKind;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    concat_commas,
    substr_edit_distance,
    ErrorContext,
    ExtraErrInfo,
    SodigyError,
    SodigyErrorKind,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct ClapError {
    kind: ClapErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl ClapError {
    pub fn invalid_flag(token: Vec<u8>, span: SpanRange) -> Self {
        match String::from_utf8(token.clone()) {
            Ok(s) => {
                // it catches the typo (this search is very expensive)
                let mut closest_flag = vec![];
                let mut closest_dist = usize::MAX;

                for flag in FLAGS.iter() {
                    let flag = flag.long();
                    let dist = substr_edit_distance(&token, flag);

                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_flag = flag.to_vec();
                    }
                }

                let mut extra = ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine);

                if token.len() > 3 && closest_dist < 3 {
                    extra.set_message(format!("Do you mean `{}`?", String::from_utf8(closest_flag).unwrap()));
                }

                ClapError {
                    kind: ClapErrorKind::InvalidFlag(s),
                    spans: smallvec![span],
                    extra,
                }
            },
            Err(_) => ClapError::invalid_utf8(span),
        }
    }

    pub fn invalid_utf8(span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn invalid_argument(kind: TokenKind, argument: &str, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::InvalidArgument(kind, argument.to_string()),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn no_args_at_all() -> Self {
        ClapError {
            kind: ClapErrorKind::NoArgsAtAll,
            spans: smallvec![],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine).set_show_span(false).to_owned(),
        }
    }

    pub fn no_arg(kind: TokenKind, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::NoArg(kind),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }
}

impl SodigyError<ClapErrorKind> for ClapError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> SpanRange {
        self.spans[0]
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &ClapErrorKind {
        &self.kind
    }

    // do we even need this?
    fn index(&self) -> u32 {
        todo!()
    }
}

pub enum ClapErrorKind {
    InvalidFlag(String),
    InvalidUtf8,
    InvalidArgument(TokenKind, String),
    NoArgsAtAll,
    NoArg(TokenKind),
}

impl SodigyErrorKind for ClapErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ClapErrorKind::InvalidFlag(s) => format!("invalid flag: `{s}`"),
            ClapErrorKind::InvalidUtf8 => String::from("invalid utf-8"),
            ClapErrorKind::NoArgsAtAll => String::from("expected an input file, got nothing"),
            ClapErrorKind::InvalidArgument(kind, arg) => format!(
                "expected {}, got `{arg}`",
                concat_commas(&kind.all_possible_values(), "or", "`", "`"),
            ),
            ClapErrorKind::NoArg(kind) => format!(
                "expected {}, got nothing",
                concat_commas(&kind.all_possible_values(), "or", "`", "`"),
            ),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ClapErrorKind::InvalidFlag(_)
            | ClapErrorKind::NoArgsAtAll => String::from("Try `sodigy --help` to see available options."),
            ClapErrorKind::InvalidUtf8
            | ClapErrorKind::InvalidArgument(_, _)
            | ClapErrorKind::NoArg(_) => String::new(),
        }
    }

    // do we even need this?
    fn index(&self) -> u32 {
        todo!()
    }
}