use crate::err::HirError;
use crate::expr::{lower_ast_expr, LocalDef};
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

pub struct Pattern {}

pub fn lower_ast_local_def(
    local_def: &ast::LocalDef,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<LocalDef, ()> {
    let let_span = local_def.let_span;
    let value = lower_ast_expr(
        &local_def.value,
        session,
        used_names,
        imports,
        name_space,
    );
    let pattern = lower_ast_pattern(
        &local_def.pattern,
        session,
    );

    Ok(LocalDef {
        let_span,
        value: value?,
        pattern: pattern?,
    })
}

pub fn lower_ast_pattern(
    pattern: &ast::Pattern,
    session: &mut HirSession,
) -> Result<Pattern, ()> {
    session.push_error(HirError::todo("pattern", pattern.span));
    Err(())
}
