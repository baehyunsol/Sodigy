use sodigy_error::UniversalError;
use std::collections::HashSet;

#[derive(Default)]
pub struct CompilerOutput {
    errors: Vec<UniversalError>,
    warnings: Vec<UniversalError>,

    // any other stuff to print
    stdout: Vec<String>,

    pub show_overall_result: bool,
    error_hashes: HashSet<u64>,
    warning_hashes: HashSet<u64>,
}

impl CompilerOutput {
    pub fn new() -> Self {
        CompilerOutput {
            errors: vec![],
            warnings: vec![],
            stdout: vec![],

            show_overall_result: true,
            error_hashes: HashSet::new(),
            warning_hashes: HashSet::new(),
        }
    }

    pub fn push_error(&mut self, mut error: UniversalError) {
        if error.is_warning {
            // TODO: write log file
            println!("FIXME: There's an internal compiler error!");

            error.is_warning = false;
        }

        if !self.error_hashes.contains(&error.hash()) {
            self.error_hashes.insert(error.hash());
            self.errors.push(error);
        }
    }

    pub fn push_warning(&mut self, mut warning: UniversalError) {
        if !warning.is_warning {
            // TODO: write log file
            println!("FIXME: There's an internal compiler error!");

            warning.is_warning = true;
        }

        if !self.warning_hashes.contains(&warning.hash()) {
            self.warning_hashes.insert(warning.hash());
            self.warnings.push(warning);
        }
    }

    pub fn dump_to_stdout(&mut self, message: String) {
        self.stdout.push(message);
    }

    pub fn merge(&mut self, other: CompilerOutput) {
        for error in other.errors.into_iter() {
            if !self.error_hashes.contains(&error.hash()) {
                self.error_hashes.insert(error.hash());
                self.errors.push(error);
            }
        }

        for warning in other.warnings.into_iter() {
            if !self.warning_hashes.contains(&warning.hash()) {
                self.warning_hashes.insert(warning.hash());
                self.warnings.push(warning);
            }
        }

        // TODO: isn't it reversing the order?
        for dump in other.stdout.into_iter() {
            self.stdout.push(dump);
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
        let mut result = vec![];
        let warnings = self.concat_warnings();
        let errors = self.concat_errors();

        if !warnings.is_empty() {
            result.push(warnings);
        }

        if !errors.is_empty() {
            result.push(errors);
        }

        if !self.stdout.is_empty() {
            result.push(self.stdout.clone().join("\n"));
        }

        if self.show_overall_result {
            let overall = format!(
                "had {} error{} and {} warning{} in total",
                self.errors.len(),
                if self.errors.len() < 2 { "" } else { "s" },
                self.warnings.len(),
                if self.warnings.len() < 2 { "" } else { "s" },
            );

            result.push(overall);
        }

        result.join("\n\n")
    }
}
