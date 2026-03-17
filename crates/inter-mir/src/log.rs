use crate::{
    ErrorContext,
    GenericCall,
    Monomorphization,
    Session,
    SolvePolyResult,
};
use sodigy_hir::Poly;
use sodigy_mir::{Func, Let, Type};
use sodigy_span::Span;

macro_rules! write_log {
    ($session:expr, $entry:expr) => {
        #[cfg(feature = "log")] {
            $session.write_log($entry);
        }
    };
}

#[derive(Clone, Debug)]
pub enum LogEntry {
    SolveSupertype {
        lhs: Type,
        rhs: Type,
        lhs_span: Option<Span>,
        rhs_span: Option<Span>,
        context: ErrorContext,
    },
    SolveFunc {
        func: Func,
        annotated_type: Type,
        infered_type: Option<Type>,
    },
    SolveLet {
        r#let: Let,
        annotated_type: Type,
        infered_type: Option<Type>,
    },
    TrySolvePoly {
        generic_call: GenericCall,
        poly_def: Option<Poly>,
        result: SolvePolyResult,
    },
    Monomorphization(Monomorphization),
}

impl Session {
    pub fn write_log(&mut self, entry: LogEntry) {
        self.log.push(entry);
    }
}

pub(crate) use write_log;
