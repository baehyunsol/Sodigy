use crate::flag::{Flag, FLAGS};
use crate::token::{Token, TokenKind};
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    concat_commas,
    substr_edit_distance,
    ErrorContext,
    ExtraErrInfo,
    RenderError,
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

                // there's no point in searching short flags
                for flag in FLAGS.iter() {
                    let flag = flag.long();
                    let dist = substr_edit_distance(&token, flag);

                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_flag = flag.to_vec();
                    }
                }

                let mut extra = ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine);

                //  --xx -> --to  (no sense)
                //  --tx -> --to  (makes sense)
                //  --verrrion -> --verrrion (makes sense)
                if (token.len() > 4 && closest_dist < 3) || (token.len() == 4 && closest_dist < 2) {
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

    pub fn no_input_files() -> Self {
        ClapError {
            kind: ClapErrorKind::NoInputFiles,
            spans: smallvec![],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine).set_show_span(false).to_owned(),
        }
    }

    pub fn multiple_input_files(span1: SpanRange, span2: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::MultipleInputFiles,
            spans: smallvec![span1, span2],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn same_flag_multiple_times(flag: Flag, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::SameFlagMultipleTimes(flag),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn unnecessary_flag(flag: Flag, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::UnnecessaryFlag(flag),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    // `start` is inclusive, and `end` is exclusive
    pub fn integer_range_error(start: u64, end: u64, given: u64, span: SpanRange) -> Self {
        assert!(start < end && (given < start || end <= given));

        ClapError {
            kind: ClapErrorKind::IntegerRangeError {
                start, end, given,
            },
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn assign_operator(
        previous_token: Option<&Token>,
        assign_operator: Token,
        next_token: Option<&Token>,
    ) -> Self {
        ClapError {
            kind: ClapErrorKind::AssignOperator {
                previous_token: previous_token.map(|t| t.clone()),
                next_token: next_token.map(|t| t.clone()),
            },
            spans: smallvec![assign_operator.span],
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

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &ClapErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        2
    }
}

pub enum ClapErrorKind {
    InvalidFlag(String),
    InvalidUtf8,
    InvalidArgument(TokenKind, String),
    NoArgsAtAll,
    NoArg(TokenKind),
    NoInputFiles,
    MultipleInputFiles,
    SameFlagMultipleTimes(Flag),
    UnnecessaryFlag(Flag),
    IntegerRangeError {
        start: u64,  // inclusive
        end: u64,    // exclusive
        given: u64,
    },
    AssignOperator {
        previous_token: Option<Token>,
        next_token: Option<Token>,
    },
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
            ClapErrorKind::NoInputFiles => String::from("no input files"),
            ClapErrorKind::MultipleInputFiles => String::from("multiple input files"),
            ClapErrorKind::SameFlagMultipleTimes(flag) => format!("`{}` given more than once", flag.render_error()),
            ClapErrorKind::UnnecessaryFlag(flag) => format!("unnecessary flag: `{}`", flag.render_error()),
            ClapErrorKind::IntegerRangeError { start, end, given } => format!(
                "expected an integer in range {start}..{end}, got {given}"
            ),
            ClapErrorKind::AssignOperator { .. } => String::from("unnecessary assign operator"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ClapErrorKind::InvalidFlag(_)
            | ClapErrorKind::NoArgsAtAll => String::from("Try `sodigy --help` to see available options."),
            ClapErrorKind::UnnecessaryFlag(flag) => format!(
                "`{}` doesn't take extra arguments",
                String::from_utf8(flag.long().to_vec()).unwrap(),
            ),
            ClapErrorKind::AssignOperator { previous_token, next_token } => {
                match (previous_token, next_token) {
                    (Some(p), Some(n)) if p.is_flag() && !n.is_flag() => format!(
                        "Try `{} {}` instead of `{} = {}`.",
                        p.render_error(),
                        n.render_error(),
                        p.render_error(),
                        n.render_error(),
                    ),
                    _ => String::new(),
                }
            },
            ClapErrorKind::InvalidUtf8
            | ClapErrorKind::InvalidArgument(_, _)
            | ClapErrorKind::NoArg(_)
            | ClapErrorKind::NoInputFiles
            | ClapErrorKind::MultipleInputFiles
            | ClapErrorKind::SameFlagMultipleTimes(_)
            | ClapErrorKind::IntegerRangeError { .. } => String::new(),
        }
    }

    // we don't need this, but I want it to look more complete
    fn index(&self) -> u32 {
        match self {
            ClapErrorKind::InvalidFlag(_) => 0,
            ClapErrorKind::InvalidUtf8 => 1,
            ClapErrorKind::InvalidArgument(_, _) => 2,
            ClapErrorKind::NoArgsAtAll => 3,
            ClapErrorKind::NoArg(_) => 4,
            ClapErrorKind::NoInputFiles => 5,
            ClapErrorKind::MultipleInputFiles => 6,
            ClapErrorKind::SameFlagMultipleTimes(_) => 7,
            ClapErrorKind::UnnecessaryFlag(_) => 8,
            ClapErrorKind::IntegerRangeError { .. } => 9,
            ClapErrorKind::AssignOperator { .. } => 10,
        }
    }
}
