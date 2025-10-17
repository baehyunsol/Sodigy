use sodigy_error::{Error, ErrorLevel, Warning};

pub trait Session {
    fn get_errors(&self) -> &[Error];
    fn get_warnings(&self) -> &[Warning];
    fn get_intermediate_dir(&self) -> &str;

    fn has_error(&self) -> bool {
        !self.get_errors().is_empty()
    }

    fn error_or_continue(&self) -> Result<(), ()> {
        if self.has_error() {
            let mut errors = vec![
                self.get_errors().to_vec(),
                self.get_warnings().to_vec(),
            ].concat();
            errors.sort_by_key(|e| (e.span, e.extra_span));
            // warnings come before errors
            errors.sort_by_key(
                |e| match ErrorLevel::from_error_kind(&e.kind) {
                    ErrorLevel::Warning => 0,
                    ErrorLevel::Error => 1,
                }
            );
            let mut stderr = vec![];
            let mut bytes = vec![];
            let mut curr_file = None;
            let mut curr_file_name = String::new();

            for error in errors.iter() {
                if error.span.get_file() != curr_file {
                    curr_file = error.span.get_file();

                    if let Some(file) = curr_file {
                        curr_file_name = file.get_name();
                        bytes = match file.read_bytes() {
                            Ok(bytes) => bytes,
                            Err(_) => {
                                curr_file = None;
                                curr_file_name = String::new();
                                vec![]
                            },
                        };
                    }

                    else {
                        curr_file_name = String::new();
                        bytes = vec![];
                    }
                }

                let level = ErrorLevel::from_error_kind(&error.kind);
                let title = match level {
                    ErrorLevel::Warning => level.color().render_fg("warning"),
                    ErrorLevel::Error => level.color().render_fg("error"),
                };
                let note = if let Some(message) = &error.extra_message {
                    format!("\nnote: {message}")
                } else {
                    String::new()
                };
                let rendered_span = if curr_file.is_some() {
                    format!("\n{}", sodigy_span::render_span(
                        &curr_file_name,
                        &bytes,
                        error.span,
                        error.extra_span,
                        sodigy_span::RenderSpanOption {
                            max_width: 88,
                            max_height: 10,
                            render_source: true,
                            color: Some(sodigy_span::ColorOption {
                                primary: level.color(),
                                secondary: sodigy_span::Color::Green,
                            }),
                        },
                    ))
                } else {
                    String::new()
                };

                stderr.push(format!(
                    "{title}: {}{note}{rendered_span}\n\n",
                    error.kind.render(self.get_intermediate_dir()),
                ));
            }

            eprintln!("{}", stderr.concat());
            Err(())
        }

        else {
            Ok(())
        }
    }
}
