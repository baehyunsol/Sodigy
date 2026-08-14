// This file is mostly copy-paste of `inter-mir/src/log.rs`.
use sodigy_error::Error;
use sodigy_hir::{Expr, Path, Pattern, Type, Use};
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
pub enum LogEntry {
    ResolveAliasStart {
        id: LogId,
    },
    ResolveAliasEnd {
        id: LogId,
        has_error: bool,
        last_errors: Vec<Error>,
    },
    ResolveAliasLoopStart(u32),
    ResolveAliasLoopEnd(u32),
    ResolveUseStart {
        id: LogId,
        r#use: Use,
    },
    ResolveUseEnd {
        id: LogId,
        r#use: Use,
        has_error: bool,
        last_errors: Vec<Error>,
    },
    ResolvePathStart {
        id: LogId,
        path: Path,
        type_args: Option<Vec<Type>>,
    },
    ResolvePathEnd {
        id: LogId,
        path: Path,
        has_error: bool,
        last_errors: Vec<Error>,
    },
    ResolveExprStart {
        id: LogId,
        expr: Expr,
    },
    ResolveExprEnd {
        id: LogId,
        expr: Expr,
        has_error: bool,
        last_errors: Vec<Error>,
    },
    ResolvePatternStart {
        id: LogId,
        pattern: Pattern,
    },
    ResolvePatternEnd {
        id: LogId,
        pattern: Pattern,
        has_error: bool,
        last_errors: Vec<Error>,
    },
    ResolveTypeStart {
        id: LogId,
        r#type: Type,
    },
    ResolveTypeEnd {
        id: LogId,
        r#type: Type,
        has_error: bool,
        last_errors: Vec<Error>,
    },
}

impl LogEntry {
    pub fn id(&self) -> LogId {
        todo!()
    }
}

pub(crate) use write_log;
