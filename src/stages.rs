use crate::ErrorsAndWarnings;
use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_endec::Endec;
use sodigy_error::SodigyError;
use sodigy_files::{file_name, global_file_session, FileError};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

type Path = String;

// TODO: nicer return type for all the stages

pub fn parse_file(
    file: &Path,
    prev_output: Option<ErrorsAndWarnings>,
    compiler_option: &CompilerOption,
) -> (Option<ParseSession>, ErrorsAndWarnings) {
    let mut errors_and_warnings = prev_output.unwrap_or_default();
    let file_session = unsafe { global_file_session() };

    let file_hash = match file_session.register_file(file) {
        Ok(f) => f,
        Err(e) => {
            errors_and_warnings.push_error(e.into());
            return (None, errors_and_warnings);
        },
    };

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
        if compiler_option.save_ir {
            let tmp_path = match generate_tmp_path(file, "tokens") {
                Ok(p) => p.to_string(),
                Err(e) => {
                    errors_and_warnings.push_error(e.into());
                    return (None, errors_and_warnings);
                },
            };

            if let Err(e) = parse_session.save_to_file(&tmp_path) {
                errors_and_warnings.push_error(e.into());
            }
        }

        parse_session.errors.clear();
        parse_session.warnings.clear();

        (Some(parse_session), errors_and_warnings)
    }
}

pub fn hir_from_tokens(
    file: &Path,
    prev_output: Option<ErrorsAndWarnings>,
    compiler_option: &CompilerOption,
) -> (Option<HirSession>, ErrorsAndWarnings) {
    let (parse_session, mut errors_and_warnings) = match IrStage::try_infer_from_ext(file) {

        // This file contains ParseSession
        Some(IrStage::Tokens) => match ParseSession::load_from_file(file) {
            Ok(parse_session) => {
                let mut errors_and_warnings = prev_output.unwrap_or_default();

                for error in parse_session.get_errors() {
                    errors_and_warnings.push_error(error.to_universal());
                }

                // TODO: do we have to escape if the session has errors?

                for warning in parse_session.get_warnings() {
                    errors_and_warnings.push_warning(warning.to_universal());
                }

                (parse_session, errors_and_warnings)
            },
            Err(e) => {
                let mut errors_and_warnings = prev_output.unwrap_or_default();
                errors_and_warnings.push_error(e.into());

                return (None, errors_and_warnings);
            },
        },
        Some(IrStage::HighIr) => {  // HirSession is already here!
            todo!()
        },

        // Let's assume it's a code file
        None => match parse_file(
            file,
            prev_output,
            compiler_option,
        ) {
            (Some(parse_session), output) => (parse_session, output),
            (None, output) => {
                return (None, output);
            },
        },
    };

    if parse_session.has_unexpanded_macros {
        // TODO: what do I do here?
        todo!();
    }

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
        if compiler_option.save_ir {
            let tmp_path = match generate_tmp_path(file, "hir") {
                Ok(p) => p.to_string(),
                Err(e) => {
                    errors_and_warnings.push_error(e.into());
                    return (None, errors_and_warnings);
                },
            };

            if let Err(e) = hir_session.save_to_file(&tmp_path) {
                errors_and_warnings.push_error(e.into());
            }
        }

        if compiler_option.dump_hir {
            println!("{}", hir_session.dump_hir());
        }

        hir_session.errors.clear();
        hir_session.warnings.clear();

        (Some(hir_session), errors_and_warnings)
    }
}

// TODO: find better place for these functions
fn generate_tmp_path(base: &Path, ext: &str) -> Result<Path, FileError> {
    // TODO: make the path more configurable
    let file_name = file_name(base)?;
    let file_hash = hash_string(base) & 0xfff_ffff;

    Ok(format!("./{file_hash:07x}_{file_name}.{ext}"))
}

fn hash_string(s: &str) -> u64 {
    let mut res = 0;

    // TODO: use a REAL hash function
    for (i, c) in s.as_bytes().iter().enumerate() {
        let n = (((i as u64 & 0x3fff) + 1) << 8) | *c as u64;
        let m = ((n * n) << 2) + n;

        res += m;
    }

    res
}
