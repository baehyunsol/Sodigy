#![deny(unused_imports)]

use sodigy_error::{SodigyError, SodigyErrorKind, UniversalError};
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_number::SodigyNumber;
use sodigy_span::SpanRange;

mod endec;

pub trait SodigySession<E: SodigyError<EK>, EK: SodigyErrorKind, W: SodigyError<WK>, WK: SodigyErrorKind, Outputs: SessionOutput<Output>, Output> {
    fn get_errors(&self) -> &Vec<E>;
    fn get_errors_mut(&mut self) -> &mut Vec<E>;

    fn get_all_errors(&self) -> Vec<UniversalError> {
        self.get_errors().iter().map(
            |e| e.to_universal()
        ).chain(self.get_previous_errors().iter().map(
            |e| e.clone()
        )).collect()
    }

    fn push_error(&mut self, e: E) {
        self.get_errors_mut().push(e);
    }

    fn pop_error(&mut self) -> Option<E> {
        self.get_errors_mut().pop()
    }

    fn has_error(&self) -> bool {
        !self.get_errors().is_empty() || !self.get_previous_errors().is_empty()
    }

    fn clear_errors(&mut self) {
        self.get_errors_mut().clear();
    }

    fn err_if_has_error(&self) -> Result<(), ()> {
        if self.has_error() {
            Err(())
        }

        else {
            Ok(())
        }
    }

    fn get_warnings(&self) -> &Vec<W>;
    fn get_warnings_mut(&mut self) -> &mut Vec<W>;

    fn get_all_warnings(&self) -> Vec<UniversalError> {
        self.get_warnings().iter().map(
            |w| w.to_universal()
        ).chain(self.get_previous_warnings().iter().map(
            |w| w.clone()
        )).collect()
    }

    fn push_warning(&mut self, w: W) {
        self.get_warnings_mut().push(w);
    }

    fn pop_warning(&mut self) -> Option<W> {
        self.get_warnings_mut().pop()
    }

    fn has_warning(&self) -> bool {
        !self.get_warnings().is_empty()
    }

    fn clear_warnings(&mut self) {
        self.get_warnings_mut().clear();
    }

    // make sure to sort errors and warnings before dumping to json
    fn sort_errors_and_warnings(&mut self) {
        self.get_errors_mut().sort_by_key(|error| error.get_first_span().unwrap_or_else(|| SpanRange::dummy()));
        self.get_warnings_mut().sort_by_key(|warning| warning.get_first_span().unwrap_or_else(|| SpanRange::dummy()));
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError>;
    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError>;
    fn get_previous_warnings(&self) -> &Vec<UniversalError>;
    fn get_previous_warnings_mut(&mut self) -> &mut Vec<UniversalError>;

    fn merge_errors_and_warnings<S: SodigySession<E_, EK_, W_, WK_, Outputs_, Output_>, E_, EK_, W_, WK_, Outputs_, Output_>(&mut self, previous_session: &S)
        where E_: SodigyError<EK_>, EK_: SodigyErrorKind, W_: SodigyError<WK_>, WK_: SodigyErrorKind, Outputs_: SessionOutput<Output_>
    {
        let self_errors = self.get_previous_errors_mut();

        for error in previous_session.get_all_errors().into_iter() {
            self_errors.push(error);
        }

        let self_warnings = self.get_previous_warnings_mut();

        for warning in previous_session.get_all_warnings().into_iter() {
            self_warnings.push(warning);
        }
    }

    fn get_all_errors_and_warnings(&self) -> Vec<UniversalError> {
        self.get_errors().iter().map(
            |err| err.to_universal()
        ).chain(self.get_warnings().iter().map(
            |warning| warning.to_universal()
        )).chain(self.get_previous_errors().iter().map(
            |err| err.clone()
        )).chain(self.get_previous_warnings().iter().map(
            |warning| warning.clone()
        )).collect()
    }

    fn get_results(&self) -> &Outputs;
    fn get_results_mut(&mut self) -> &mut Outputs;

    fn push_result(&mut self, result: Output) {
        self.get_results_mut().push(result);
    }

    fn pop_result(&mut self) -> Option<Output> {
        self.get_results_mut().pop()
    }

    fn clear_results(&mut self) {
        self.get_results_mut().clear();
    }

    // immutable interner cannot do anything
    fn get_interner(&mut self) -> &mut InternSession;
    fn get_interner_cloned(&self) -> InternSession;

    fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        self.get_interner().intern_string(string)
    }

    fn unintern_string_fast(&mut self, string: InternedString) -> Option<&[u8]> {
        self.get_interner().unintern_string_fast(string)
    }

    fn unintern_string(&mut self, string: InternedString) -> &[u8] {
        self.get_interner().unintern_string(string)
    }

    fn intern_numeric(&mut self, n: SodigyNumber) -> InternedNumeric {
        self.get_interner().intern_numeric(n)
    }

    fn unintern_numeric(&mut self, s: InternedNumeric) -> &SodigyNumber {
        self.get_interner().unintern_numeric(s)
    }

    fn get_snapshots_mut(&mut self) -> &mut Vec<SessionSnapshot>;

    fn take_snapshot(&mut self) {
        let snapshot = SessionSnapshot {
            errors: self.get_errors().len(),
            warnings: self.get_warnings().len(),
            results: self.get_results().len(),
        };

        self.get_snapshots_mut().push(snapshot);
    }

    // there's no point in returning the snapshot. It only tells the caller whether
    // self.snapshots is empty or not
    fn pop_snapshot(&mut self) -> Result<(), ()> {
        self.get_snapshots_mut().pop().map(|_| ()).ok_or(())
    }

    fn restore_to_last_snapshot(&mut self) {
        let last_snapshot = self.get_snapshots_mut().pop().unwrap();

        while self.get_errors().len() > last_snapshot.errors {
            self.get_errors_mut().pop().unwrap();
        }

        while self.get_warnings().len() > last_snapshot.warnings {
            self.get_warnings_mut().pop().unwrap();
        }

        while self.get_results().len() > last_snapshot.results {
            self.get_results_mut().pop().unwrap();
        }
    }

    fn get_compiler_option(&self) -> &CompilerOption;
}

pub trait SessionOutput<T> {
    fn pop(&mut self) -> Option<T>;
    fn push(&mut self, v: T);
    fn clear(&mut self);
    fn len(&self) -> usize;
}

impl<T> SessionOutput<T> for Vec<T> {
    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, v: T) {
        self.push(v);
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// for now, it only stores lengths, not the contents
// the implementation has to be changed when the logic
// gets more complicated
#[derive(Clone)]
pub struct SessionSnapshot {
    errors: usize,
    warnings: usize,
    results: usize,
}

use sodigy_config::CompilerOption;

// don't call these
impl SessionOutput<CompilerOption> for CompilerOption {
    fn pop(&mut self) -> Option<CompilerOption> { unreachable!() }
    fn push(&mut self, _: CompilerOption) { unreachable!() }
    fn clear(&mut self) { unreachable!() }
    fn len(&self) -> usize { unreachable!() }
}
