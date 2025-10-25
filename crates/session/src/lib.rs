use sodigy_error::{Error, ErrorLevel, Warning};
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    Span,
    render_spans,
};

pub trait Session {
    fn get_errors(&self) -> &[Error];
    fn get_warnings(&self) -> &[Warning];
    fn get_intermediate_dir(&self) -> &str;

    fn has_error(&self) -> bool {
        !self.get_errors().is_empty()
    }

    fn continue_or_dump_errors(&self) -> Result<(), ()> {
        if self.has_error() {
            dump_errors(
                vec![
                    self.get_errors().to_vec(),
                    self.get_warnings().to_vec(),
                ].concat(),
                self.get_intermediate_dir(),
            );
            Err(())
        }

        else {
            Ok(())
        }
    }

    fn dump_warnings(&self) {
        dump_errors(
            self.get_warnings().to_vec(),
            self.get_intermediate_dir(),
        );
    }
}

fn dump_errors(mut errors: Vec<Error>, intermediate_dir: &str) {
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
