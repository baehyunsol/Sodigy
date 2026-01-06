use crate::{Error, ErrorLevel};
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    Span,
    render_spans,
};
use std::collections::HashSet;

pub fn deduplicate(errors: &mut Vec<Error>) -> Vec<Error> {
    errors.drain(..).collect::<HashSet<_>>().into_iter().collect()
}

pub fn dump(mut errors: Vec<Error>, intermediate_dir: &str) {
    errors.sort_by_key(|e| e.spans.get(0).map(|s| s.span).unwrap_or(Span::None));
    // warnings come before errors
    errors.sort_by_key(
        |e| match ErrorLevel::from_error_kind(&e.kind) {
            ErrorLevel::Warning => 0,
            ErrorLevel::Error => 1,
        }
    );
    let mut stderr = vec![];
    let mut session = RenderSpanSession::new(intermediate_dir);

    for error in errors.iter() {
        let level = ErrorLevel::from_error_kind(&error.kind);
        let title = match level {
            ErrorLevel::Warning => level.color().render_fg("warning"),
            ErrorLevel::Error => level.color().render_fg("error"),
        };
        let note = if let Some(note) = &error.note {
            format!("\nnote: {note}")
        } else {
            String::new()
        };
        let rendered_span = format!("\n{}", render_spans(
            &error.spans,
            &RenderSpanOption {
                max_width: 88,
                max_height: 10,
                context: 2,
                render_source: true,
                color: Some(ColorOption {
                    primary: level.color(),
                    auxiliary: Color::Blue,
                    info: Color::Green,
                }),
                group_delim: None,
            },
            &mut session,
        ));

        stderr.push(format!(
            "{title}: {}{note}{rendered_span}\n\n",
            error.kind.render(intermediate_dir),
        ));
    }

    eprintln!("{}", stderr.concat());
}
