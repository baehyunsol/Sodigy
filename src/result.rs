use sodigy_error::UniversalError;
use std::collections::HashSet;

#[derive(Default)]
pub struct ErrorsAndWarnings {
    errors: Vec<UniversalError>,
    warnings: Vec<UniversalError>,

    error_hashes: HashSet<u64>,
    warning_hashes: HashSet<u64>,
}

impl ErrorsAndWarnings {
    pub fn new() -> Self {
        ErrorsAndWarnings {
            errors: vec![],
            warnings: vec![],

            error_hashes: HashSet::new(),
            warning_hashes: HashSet::new(),
        }
    }

    pub fn push_error(&mut self, error: UniversalError) {
        if !self.error_hashes.contains(&error.hash()) {
            self.error_hashes.insert(error.hash());
            self.errors.push(error);
        }
    }

    pub fn push_warning(&mut self, warning: UniversalError) {
        if !self.warning_hashes.contains(&warning.hash()) {
            self.warning_hashes.insert(warning.hash());
            self.warnings.push(warning);
        }
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn concat_errors(&mut self) -> String {
        self.errors.sort_by_key(|w| w.first_span());

        self.errors.iter().map(
            |e| e.rendered().to_string()
        ).collect::<Vec<String>>().join("\n\n")
    }

    pub fn concat_warnings(&mut self) -> String {
        self.warnings.sort_by_key(|w| w.first_span());

        self.warnings.iter().map(
            |w| w.rendered().to_string()
        ).collect::<Vec<String>>().join("\n\n")
    }

    pub fn concat_results(&mut self) -> String {
        vec![
            self.concat_warnings(),
            self.concat_errors(),
            format!(
                "had {} error{} and {} warning{} in total",
                self.errors.len(),
                if self.errors.len() < 2 { "" } else { "s" },
                self.warnings.len(),
                if self.warnings.len() < 2 { "" } else { "s" },
            ),
        ].join("\n\n")
    }
}
