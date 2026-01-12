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
        }
    }
}

pub fn dump_errors(
    mut errors: Vec<Error>,
    mut warnings: Vec<Error>,
    intermediate_dir: &str,
    option: DumpErrorOption,
    elapsed_ms: Option<u64>,  // may or may not be available
) {
    errors.sort_by_key(|e| e.spans.get(0).map(|s| s.span).unwrap_or(Span::None));
    warnings.sort_by_key(|w| w.spans.get(0).map(|s| s.span).unwrap_or(Span::None));

    let mut stderr = vec![];
    let mut session = RenderSpanSession::new(intermediate_dir);

    // warnings come before errors
    // We don't use `ErrorLevel::from_error_kind` anymore because I want to implement `#[deny(_)]` someday.
    for (error, level) in warnings.iter().map(|w| (w, ErrorLevel::Warning)).chain(errors.iter().map(|e| (e, ErrorLevel::Error))) {
        let color = match level {
            ErrorLevel::Error => option.error_color,
            ErrorLevel::Warning => option.warning_color,
        };
        let title = match level {
            ErrorLevel::Error => format!("error (e-{:04})", error.kind.index()),
            ErrorLevel::Warning => format!("warning (w-{:04})", error.kind.index()),
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

    eprint!("{}", stderr.join(&option.delim));

    if !stderr.is_empty() {
        eprint!("{}", option.delim);
    }

    eprintln!(
        "Finished: {} error{} and {} warning{}{}",
        errors.len(),
        if errors.len() == 1 { "" } else { "s" },
        warnings.len(),
        if warnings.len() == 1 { "" } else { "s" },
        match elapsed_ms {
            Some(elapsed_ms) => format!(
                " (elapsed {}.{:03}s)",
                elapsed_ms / 1000,
                elapsed_ms % 1000,
            ),
            None => String::new(),
        },
    );
}
