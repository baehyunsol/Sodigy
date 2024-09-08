#![deny(unused_imports)]

use log::error;
use sodigy_error::{SodigyError, SodigyErrorKind, UniversalError};
use sodigy_session::{SessionOutput, SodigySession};
use std::collections::HashSet;

#[derive(Default)]
pub struct CompilerOutput {
    errors: Vec<UniversalError>,
    warnings: Vec<UniversalError>,

    // any other stuff to print
    stdout: Vec<String>,

    // shows "had _ error and _ warning in total"
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

            show_overall_result: false,
            error_hashes: HashSet::new(),
            warning_hashes: HashSet::new(),
        }
    }

    pub fn collect_errors_and_warnings_from_session<S, E, W, O1, O2, Ek, Wk>(&mut self, session: &S)
    where
        S: SodigySession<E, Ek, W, Wk, O1, O2>,
        E: SodigyError<Ek>,
        W: SodigyError<Wk>,
        O1: SessionOutput<O2>,
        Ek: SodigyErrorKind,
        Wk: SodigyErrorKind,
    {
        for error in session.get_all_errors_and_warnings() {
            if error.is_warning {
                self.push_warning(error);
            }

            else {
                self.push_error(error);
            }
        }
    }

    pub fn push_error(&mut self, mut error: UniversalError) {
        if error.is_warning {
            error!("push_error(e) where `e` is not an error");

            error.is_warning = false;
        }

        if !self.error_hashes.contains(&error.hash()) {
            self.error_hashes.insert(error.hash());
            self.errors.push(error);
        }
    }

    pub fn push_warning(&mut self, mut warning: UniversalError) {
        if !warning.is_warning {
            error!("push_warning(e) where `e` is not a warning");

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

        if self.show_overall_result || other.show_overall_result {
            self.show_overall_result = true;
        }
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    /// It requires `&mut self` because it has to
    /// sort errors.
    pub fn concat_errors(&mut self) -> String {
        self.errors.sort_by_key(|w| w.first_span());

        self.errors.iter().map(
            |e| e.rendered().to_string()
        ).collect::<Vec<String>>().join("\n\n")
    }

    /// It requires `&mut self` because it has to
    /// sort warnings.
    pub fn concat_warnings(&mut self) -> String {
        self.warnings.sort_by_key(|w| w.first_span());

        self.warnings.iter().map(
            |w| w.rendered().to_string()
        ).collect::<Vec<String>>().join("\n\n")
    }

    /// It requires `&mut self` because it has to
    /// sort errors and warnings.
    pub fn concat_results(&mut self) -> (String, String) {  // (stdout, stderr)
        let mut stdout = vec![];
        let mut stderr = vec![];
        let warnings = self.concat_warnings();
        let errors = self.concat_errors();

        if !warnings.is_empty() {
            stderr.push(warnings);
        }

        if !errors.is_empty() {
            stderr.push(errors);
        }

        if !self.stdout.is_empty() {
            stdout.push(self.stdout.clone().join("\n"));
        }

        if self.show_overall_result {
            let overall = format!(
                "Complete: had {} error{} and {} warning{} in total",
                self.errors.len(),
                if self.errors.len() < 2 { "" } else { "s" },
                self.warnings.len(),
                if self.warnings.len() < 2 { "" } else { "s" },
            );

            stderr.push(overall);
        }

        (stdout.join("\n\n"), stderr.join("\n\n"))
    }

    pub fn concat_and_dump_results(&mut self) {
        let (stdout, stderr) = self.concat_results();

        println!("{stdout}");
        eprintln!("{stderr}");
    }
}
