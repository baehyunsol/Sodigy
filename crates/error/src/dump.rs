use crate::{Error, ErrorLevel};
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    Span,
    render_spans,
};

/// Set colors to `Color::None` if you want to disable all the colors.
#[derive(Clone, Debug)]
pub struct DumpErrorOption {
    pub error_color: Color,
    pub warning_color: Color,
    pub auxiliary_color: Color,
    pub info_color: Color,
    pub delim: String,
    pub span_max_width: usize,
    pub span_max_height: usize,
    pub span_context: usize,
    pub max_dump: Option<usize>,
}

impl Default for DumpErrorOption {
    fn default() -> DumpErrorOption {
        DumpErrorOption {
            error_color: Color::Red,
            warning_color: Color::Yellow,
            auxiliary_color: Color::Blue,
            info_color: Color::Green,
            delim: String::from("\n\n---------\n\n"),
            span_max_width: 88,
            span_max_height: 10,
            span_context: 2,
            max_dump: Some(100),
        }
    }
}

#[must_use]
pub fn dump_errors(
    mut errors: Vec<Error>,
    mut warnings: Vec<Error>,
    intermediate_dir: &str,
    option: DumpErrorOption,
    elapsed_ms: Option<u64>,  // may or may not be available
    show_summary: bool,
) -> String {
    errors.sort_by_key(|e| e.spans.get(0).map(|s| s.span.clone()).unwrap_or(Span::None));
    warnings.sort_by_key(|w| w.spans.get(0).map(|s| s.span.clone()).unwrap_or(Span::None));

    let total_errors = errors.len();
    let mut truncated_errors = None;
    let total_warnings = warnings.len();
    let mut truncated_warnings = None;

    if let Some(n) = option.max_dump && errors.len() > n {
        truncated_errors = Some(errors.len() - n);
        errors = errors.drain(..).take(n).collect();
    }

    if let Some(n) = option.max_dump && warnings.len() > n {
        truncated_warnings = Some(warnings.len() - n);
        warnings = warnings.drain(..).take(n).collect();
    }

    let mut stderr = vec![];
    let mut session = RenderSpanSession::new(intermediate_dir);

    // warnings come before errors
    // We don't use `ErrorLevel::from_error_kind` anymore because I want to implement `#[deny(_)]` someday.
    for (error, level) in warnings.iter().map(|w| (w, ErrorLevel::Warning)).chain(errors.iter().map(|e| (e, ErrorLevel::Error))) {
        let color = match level {
            ErrorLevel::Error => option.error_color,
            ErrorLevel::Warning => option.warning_color,
            ErrorLevel::Lint => unreachable!(),
        };
        let title = match level {
            ErrorLevel::Error => format!("error (e-{:04})", error.kind.index()),
            ErrorLevel::Warning => format!("warning (w-{:04})", error.kind.index()),
            ErrorLevel::Lint => unreachable!(),
        };
        let colored_title = color.render_fg(&title);
        let note = if let Some(note) = &error.note {
            format!("\nnote: {note}")
        } else {
            String::new()
        };
        let rendered_span = format!("\n{}", render_spans(
            &error.spans,
            &RenderSpanOption {
                max_width: option.span_max_width,
                max_height: option.span_max_height,
                context: option.span_context,
                render_source: true,
                color: Some(ColorOption {
                    primary: color,
                    auxiliary: option.auxiliary_color,
                    info: option.info_color,
                }),
                group_delim: None,
            },
            &mut session,
        ));

        stderr.push(format!(
            "{colored_title}: {}{note}{rendered_span}",
            error.kind.render(intermediate_dir),
        ));
    }

    if show_summary {
        let error_truncation_note = if let Some(n) = truncated_errors {
            format!(
                "\nNote: There are too many errors, so only the first {} errors are shown ({n} error{} truncated).",
                option.max_dump.unwrap(),
                if n == 1 { "" } else { "s" },
            )
        } else {
            String::new()
        };
        let warning_truncation_note = if let Some(n) = truncated_warnings {
            format!(
                "\nNote: There are too many warnings, so only the first {} warnings are shown ({n} warning{} truncated).",
                option.max_dump.unwrap(),
                if n == 1 { "" } else { "s" },
            )
        } else {
            String::new()
        };

        stderr.push(format!(
            "Finished: {total_errors} error{} and {total_warnings} warning{}{}{error_truncation_note}{warning_truncation_note}\n",
            if total_errors == 1 { "" } else { "s" },
            if total_warnings == 1 { "" } else { "s" },
            match elapsed_ms {
                Some(elapsed_ms) => format!(
                    " (elapsed {}.{:03}s)",
                    elapsed_ms / 1000,
                    elapsed_ms % 1000,
                ),
                None => String::new(),
            },
        ));
    }

    stderr.join(&option.delim)
}
