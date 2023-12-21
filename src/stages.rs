use crate::ErrorsAndWarnings;
use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_endec::Endec;
use sodigy_error::SodigyError;
use sodigy_files::{global_file_session, FileHash};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

// TODO: nicer return type for all the stages

pub fn parse_stage(
    file_hash: FileHash,
    prev_output: Option<ErrorsAndWarnings>,
    save_file_to: Option<String>,
) -> (Option<ParseSession>, ErrorsAndWarnings) {
    let mut errors_and_warnings = prev_output.unwrap_or_default();
    let file_session = unsafe { global_file_session() };
    let input = match file_session.get_file_content(file_hash) {
        Ok(f) => f,
        Err(e) => {
            errors_and_warnings.push_error(e.into());

            return (None, errors_and_warnings);
        },
    };

    let mut lex_session = LexSession::new();

    if let Err(()) = lex(input, 0, SpanPoint::at_file(file_hash, 0), &mut lex_session) {
        for error in lex_session.get_errors() {
            errors_and_warnings.push_error(error.to_universal());
        }

        return (None, errors_and_warnings);
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_tokens();
    let mut new_lex_session = LexSession::new();

    let res = from_tokens(tokens, &mut parse_session, &mut new_lex_session);

    for warning in parse_session.get_warnings() {
        errors_and_warnings.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in parse_session.get_errors() {
            errors_and_warnings.push_error(error.to_universal());
        }

        for error in new_lex_session.get_errors() {
            errors_and_warnings.push_error(error.to_universal());
        }

        (None, errors_and_warnings)
    }

    else {
        if let Some(path) = save_file_to {
            if let Err(e) = parse_session.save_to_file(&path) {
                errors_and_warnings.push_error(e.into());
            }
        }

        parse_session.errors.clear();
        parse_session.warnings.clear();

        (Some(parse_session), errors_and_warnings)
    }
}

pub fn hir_stage(
    parse_session: &ParseSession,
    prev_output: Option<ErrorsAndWarnings>,
    save_file_to: Option<String>,
) -> (Option<HirSession>, ErrorsAndWarnings) {
    if parse_session.has_unexpanded_macros {
        // TODO: what do I do here?
        todo!();
    }

    let mut errors_and_warnings = prev_output.unwrap_or_default();

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    let mut tokens = parse_session.get_tokens().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);
    let res = parse_stmts(&mut tokens, &mut ast_session);

    for warning in ast_session.get_warnings() {
        errors_and_warnings.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in ast_session.get_errors() {
            errors_and_warnings.push_error(error.to_universal());
        }

        return (None, errors_and_warnings);
    }

    let mut hir_session = HirSession::new();
    let res = lower_stmts(ast_session.get_stmts(), &mut hir_session);

    for warning in hir_session.get_warnings() {
        errors_and_warnings.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in hir_session.get_errors() {
            errors_and_warnings.push_error(error.to_universal());
        }

        (None, errors_and_warnings)
    }

    else {
        if let Some(path) = save_file_to {
            if let Err(e) = hir_session.save_to_file(&path) {
                errors_and_warnings.push_error(e.into());
            }
        }

        hir_session.errors.clear();
        hir_session.warnings.clear();

        (Some(hir_session), errors_and_warnings)
    }
}
