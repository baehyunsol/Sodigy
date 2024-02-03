#![deny(unused_imports)]

use sodigy_files::last_modified;
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_number::SodigyNumber;

mod endec;

pub trait SodigySession<E, W, Outputs: SessionOutput<Output>, Output> {
    fn get_errors(&self) -> &Vec<E>;
    fn get_errors_mut(&mut self) -> &mut Vec<E>;

    fn push_error(&mut self, e: E) {
        self.get_errors_mut().push(e);
    }

    fn pop_error(&mut self) -> Option<E> {
        self.get_errors_mut().pop()
    }

    fn has_error(&self) -> bool {
        !self.get_errors().is_empty()
    }

    fn clear_errors(&mut self) {
        self.get_errors_mut().clear();
    }

    fn err_if_has_err(&self) -> Result<(), ()> {
        if self.has_error() {
            Err(())
        }

        else {
            Ok(())
        }
    }

    fn get_warnings(&self) -> &Vec<W>;
    fn get_warnings_mut(&mut self) -> &mut Vec<W>;

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

    fn unintern_string(&mut self, string: InternedString) -> Option<&[u8]> {
        self.get_interner().unintern_string(string)
    }

    fn intern_numeric(&mut self, n: SodigyNumber) -> InternedNumeric {
        self.get_interner().intern_numeric(n)
    }

    fn unintern_numeric(&mut self, s: InternedNumeric) -> Option<&SodigyNumber> {
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

    fn get_dependencies(&self) -> &Vec<SessionDependency>;
    fn get_dependencies_mut(&mut self) -> &mut Vec<SessionDependency>;

    fn add_dependency(&mut self, dependency: SessionDependency) {
        self.get_dependencies_mut().push(dependency);
    }

    fn check_all_dependency_up_to_date(&self) -> bool {
        for SessionDependency { path, last_modified_at } in self.get_dependencies().iter() {
            if let Ok(last_modified_at_) = last_modified(path) {
                if *last_modified_at != last_modified_at_ {
                    return false;
                }
            }

            else {
                return false;
            }
        }

        true
    }
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

#[derive(Clone)]
pub struct SessionDependency {
    pub path: String,
    pub last_modified_at: u64,  // hash of st_mtime
}
