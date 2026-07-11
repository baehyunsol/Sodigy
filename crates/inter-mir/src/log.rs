use crate::{
    AssociatedFuncInstance,
    ErrorContext,
    GenericCall,
    Monomorphization,
    PolySolver,
    SolvePolyResult,
    TypeError,
};
use sodigy_error::Error;
use sodigy_hir::{EnumShape, Poly, StructShape};
use sodigy_mir::{Assert, Expr, Func, Let, Type};
use sodigy_parse::Field;
use sodigy_span::Span;
use std::sync::atomic::{AtomicU32, Ordering};

macro_rules! write_log {
    ($session:expr, $entry:expr) => {
        #[cfg(feature = "log")] {
            $session.log.push($entry);
        }
    };
}

// VIBE NOTE: gpt-5.5 (via neukgu-chat) wrote this atomic increment.
static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogId(u32);

impl LogId {
    pub fn new() -> Self {
        LogId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug)]
pub enum BlockedTypeVarKind {
    CallingTypeVar {
        expr: Expr,
        type_var: Type,
    },
    FieldOfTypeVar {
        field: Vec<Field>,
        type_var: Type,
    },
}

// Many functions return `Err(())` when there's an error. The actual error is
// stored in the session. So the entries have `last_errors` field, which stores
// the most recent 3 errors. The errors may or may not be from the logged function.
#[derive(Clone, Debug)]
pub enum LogEntry {
    TypeSolveLoopStart(u32),
    TypeSolveLoopEnd(u32),
    SolveSupertypeStart {
        id: LogId,
        lhs: Type,
        rhs: Type,
        lhs_span: Option<Span>,
        rhs_span: Option<Span>,
        context: ErrorContext,
    },
    SolveSupertypeEnd {
        id: LogId,
        solved_type: Option<Type>,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    SolveFuncStart {
        id: LogId,
        func: Func,
    },
    SolveFuncEnd {
        id: LogId,
        annotated_type: Type,
        infered_type: Option<Type>,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    SolveLetStart {
        id: LogId,
        r#let: Let,
    },
    SolveLetEnd {
        id: LogId,
        annotated_type: Type,
        infered_type: Option<Type>,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    SolveAssertStart {
        id: LogId,
        assert: Assert,
    },
    SolveAssertEnd {
        id: LogId,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    SolveExprStart {
        id: LogId,
        expr: Expr,
    },
    SolveExprEnd {
        id: LogId,
        infered_type: Option<Type>,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    GetTypeOfFieldStart {
        id: LogId,
        r#type: Type,
        field: Vec<Field>,
    },
    GetTypeOfFieldEnd {
        id: LogId,
        associated_func: Option<AssociatedFuncInstance>,
        infered_type: Option<Type>,
        has_error: bool,

        // `get_type_of_field` returns the exact error, so this vector has 0 or 1 errors.
        last_errors: Vec<(TypeError, Error)>,
    },
    GetItemShapeStart {
        id: LogId,
        r#type: Type,
        def_span: Span,
    },
    GetItemShapeEnd {
        id: LogId,
        struct_shape: Option<StructShape>,
        enum_shape: Option<EnumShape>,
    },
    InitPolySolverStart {
        id: LogId,
        poly_def_span: Span,
        poly: Poly,
    },
    InitPolySolverEnd {
        id: LogId,
        solver: Option<PolySolver>,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    InitPolySolversStart {
        id: LogId,
    },
    InitPolySolversEnd {
        id: LogId,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    TrySolvePolyStart {
        id: LogId,
        generic_call: GenericCall,
        poly: Option<Poly>,
        solver: Option<PolySolver>,
    },
    TrySolvePolyEnd {
        id: LogId,
        result: SolvePolyResult,
    },
    MonomorphizeFuncStart {
        id: LogId,
        func: Func,
        monomorphization: Monomorphization,
    },
    MonomorphizeFuncEnd {
        id: LogId,
        result: Func,
    },
    CheckAllTypesInferedStart {
        id: LogId,
    },
    CheckAllTypesInferedEnd {
        id: LogId,
        has_error: bool,
        last_errors: Vec<(TypeError, Error)>,
    },
    Monomorphization(Monomorphization),
    BlockedTypeVar {
        kind: BlockedTypeVarKind,
        span: Span,
    },
}

impl LogEntry {
    pub fn id(&self) -> Option<LogId> {
        match self {
            LogEntry::TypeSolveLoopStart(n) | LogEntry::TypeSolveLoopEnd(n) => Some(LogId(0x4_0000 | n)),
            LogEntry::SolveSupertypeStart { id, .. } |
            LogEntry::SolveSupertypeEnd { id, .. } |
            LogEntry::SolveFuncStart { id, .. } |
            LogEntry::SolveFuncEnd { id, .. } |
            LogEntry::SolveLetStart { id, .. } |
            LogEntry::SolveLetEnd { id, .. } |
            LogEntry::SolveAssertStart { id, .. } |
            LogEntry::SolveAssertEnd { id, .. } |
            LogEntry::SolveExprStart { id, .. } |
            LogEntry::SolveExprEnd { id, .. } |
            LogEntry::GetTypeOfFieldStart { id, .. } |
            LogEntry::GetTypeOfFieldEnd { id, .. } |
            LogEntry::GetItemShapeStart { id, .. } |
            LogEntry::GetItemShapeEnd { id, .. } |
            LogEntry::InitPolySolverStart { id, .. } |
            LogEntry::InitPolySolverEnd { id, .. } |
            LogEntry::InitPolySolversStart { id, .. } |
            LogEntry::InitPolySolversEnd { id, .. } |
            LogEntry::TrySolvePolyStart { id, .. } |
            LogEntry::TrySolvePolyEnd { id, .. } |
            LogEntry::MonomorphizeFuncStart { id, .. } |
            LogEntry::MonomorphizeFuncEnd { id, .. } |
            LogEntry::CheckAllTypesInferedStart { id, .. } |
            LogEntry::CheckAllTypesInferedEnd { id, .. } => Some(*id),
            LogEntry::Monomorphization(_) |
            LogEntry::BlockedTypeVar { .. } => None,
        }
    }
}

pub(crate) use write_log;
