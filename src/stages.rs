use crate::{CompilerOutput, SAVE_IRS_AT};
use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_endec::{Endec, EndecError, EndecErrorContext, EndecErrorKind};
use sodigy_error::SodigyError;
use sodigy_files::{
    create_dir,
    exists,
    file_name,
    global_file_session,
    is_dir,
    join,
    last_modified,
    parent,
    read_bytes,
    write_string,
    FileError,
    FileErrorContext,
    WriteMode,
};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

type Path = String;

// TODO: this file has a lot of duplicate code blocks
// TODO: nicer return type for all the stages
// TODO: remove all the hard-coded strings: "tokens", "hir", ...
// TODO: `parse_file` and `hir_from_tokens` are very similar...
// TODO: make all the stages able to deal with raw inputs, not just files

pub fn parse_file(
    file: &Path,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<ParseSession>, CompilerOutput) {
    let mut compiler_output = prev_output.unwrap_or_default();

    if let Some(s) = try_construct_session_from_saved_ir::<ParseSession>(file, "tokens") {
        match s {
            Ok(session) => {
                // TODO: this pattern is used over and over...
                for error in session.get_errors() {
                    compiler_output.push_error(error.to_universal());
                }

                for warning in session.get_warnings() {
                    compiler_output.push_warning(warning.to_universal());
                }

                // TODO: this if statement is duplicate
                if compiler_option.dump_tokens {
                    let res = session.dump_tokens();

                    if let Some(path) = &compiler_option.dump_tokens_to {
                        if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                            compiler_output.push_error(e.set_context(FileErrorContext::DumpingTokensToFile).to_owned().into());
                        }
                    }

                    else {
                        compiler_output.dump_to_stdout(res);
                    }
                }

                return (Some(session), compiler_output);
            },
            Err(e) => {
                // TODO: warn the programmer that something's wrong with the incremental compilation
                // there's nothing wrong in the compilation, tho
            },
        }
    }

    let file_session = unsafe { global_file_session() };

    let file_hash = match file_session.register_file(file) {
        Ok(f) => f,
        Err(e) => {
            compiler_output.push_error(e.into());
            return (None, compiler_output);
        },
    };

    let input = match file_session.get_file_content(file_hash) {
        Ok(f) => f,
        Err(e) => {
            compiler_output.push_error(e.into());

            return (None, compiler_output);
        },
    };

    let mut lex_session = LexSession::new();

    if let Err(()) = lex(input, 0, SpanPoint::at_file(file_hash, 0), &mut lex_session) {
        for error in lex_session.get_errors() {
            compiler_output.push_error(error.to_universal());
        }

        return (None, compiler_output);
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_tokens();
    let mut new_lex_session = LexSession::new();

    let res = from_tokens(tokens, &mut parse_session, &mut new_lex_session);

    for warning in parse_session.get_warnings() {
        compiler_output.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in parse_session.get_errors() {
            compiler_output.push_error(error.to_universal());
        }

        for error in new_lex_session.get_errors() {
            compiler_output.push_error(error.to_universal());
        }

        (None, compiler_output)
    }

    else {
        if compiler_option.save_ir {
            let tmp_path = match generate_path_for_ir(file, "tokens", true) {
                Ok(p) => p.to_string(),
                Err(e) => {
                    compiler_output.push_error(e.into());
                    return (None, compiler_output);
                },
            };

            let file_metadata = match last_modified(file) {
                Ok(m) => m.max(1),  // let's avoid 0 -> see the Err(e) branch
                Err(e) => {
                    // TODO: warn the programmer that incremental compilation is not working
                    // compiler_output.push_warning(...);

                    0
                },
            };

            if let Err(mut e) = parse_session.save_to_file(&tmp_path, Some(file_metadata)) {
                compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
            }
        }

        if compiler_option.dump_tokens {
            let res = parse_session.dump_tokens();

            if let Some(path) = &compiler_option.dump_tokens_to {
                if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                    compiler_output.push_error(e.set_context(FileErrorContext::DumpingTokensToFile).to_owned().into());
                }
            }

            else {
                compiler_output.dump_to_stdout(res);
            }
        }

        parse_session.errors.clear();
        parse_session.warnings.clear();

        (Some(parse_session), compiler_output)
    }
}

pub fn hir_from_tokens(
    file: &Path,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<HirSession>, CompilerOutput) {
    if let Some(s) = try_construct_session_from_saved_ir::<HirSession>(file, "hir") {
        match s {
            Ok(session) => {
                let mut compiler_output = prev_output.unwrap_or_default();

                // TODO: this pattern is used over and over...
                for error in session.get_errors() {
                    compiler_output.push_error(error.to_universal());
                }

                for warning in session.get_warnings() {
                    compiler_output.push_warning(warning.to_universal());
                }

                // TODO: this if statement is duplicate
                if compiler_option.dump_hir {
                    let res = session.dump_hir();

                    if let Some(path) = &compiler_option.dump_hir_to {
                        if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                            compiler_output.push_error(e.set_context(FileErrorContext::DumpingHirToFile).to_owned().into());
                        }
                    }

                    else {
                        compiler_output.dump_to_stdout(res);
                    }
                }

                return (Some(session), compiler_output);
            },
            Err(e) => {
                // TODO: warn the programmer that something's wrong with the incremental compilation
                // there's nothing wrong in the compilation, tho
            },
        }
    }

    let (parse_session, mut compiler_output) = match IrStage::try_infer_from_ext(file) {

        // This file contains ParseSession
        Some(IrStage::Tokens) => match ParseSession::load_from_file(file, None) {
            Ok(parse_session) => {
                let mut compiler_output = prev_output.unwrap_or_default();
                let mut has_error = false;

                for error in parse_session.get_errors() {
                    compiler_output.push_error(error.to_universal());
                    has_error = true;
                }

                for warning in parse_session.get_warnings() {
                    compiler_output.push_warning(warning.to_universal());
                }

                // We don't allow an erroneous session to continue compilation
                if has_error {
                    return (None, compiler_output);
                }

                (parse_session, compiler_output)
            },
            Err(e) => {
                let mut compiler_output = prev_output.unwrap_or_default();
                compiler_output.push_error(e.into());

                if is_human_readable(file) {
                    compiler_output.push_error(
                        EndecError::human_readable_file("--dump-tokens", file)
                            .set_context(EndecErrorContext::ConstructingTokensFromIr).to_owned().into()
                    );
                }

                return (None, compiler_output);
            },
        },
        Some(IrStage::HighIr) => match HirSession::load_from_file(file, None) {  // HirSession is already here!
            Ok(hir_session) => {
                let mut compiler_output = prev_output.unwrap_or_default();

                for error in hir_session.get_errors() {
                    compiler_output.push_error(error.to_universal());
                }

                for warning in hir_session.get_warnings() {
                    compiler_output.push_warning(warning.to_universal());
                }

                return (Some(hir_session), compiler_output);
            },
            Err(e) => {
                let mut compiler_output = prev_output.unwrap_or_default();
                compiler_output.push_error(e.into());

                if is_human_readable(file) {
                    compiler_output.push_error(
                        EndecError::human_readable_file("--dump-hir", file)
                            .set_context(EndecErrorContext::ConstructingHirFromIr).to_owned().into()
                    );
                }

                return (None, compiler_output);
            },
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
        compiler_output.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in ast_session.get_errors() {
            compiler_output.push_error(error.to_universal());
        }

        return (None, compiler_output);
    }

    let mut hir_session = HirSession::new();
    let res = lower_stmts(ast_session.get_stmts(), &mut hir_session);

    for warning in hir_session.get_warnings() {
        compiler_output.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in hir_session.get_errors() {
            compiler_output.push_error(error.to_universal());
        }

        (None, compiler_output)
    }

    else {
        if compiler_option.save_ir {
            let tmp_path = match generate_path_for_ir(file, "hir", true) {
                Ok(p) => p.to_string(),
                Err(e) => {
                    compiler_output.push_error(e.into());
                    return (None, compiler_output);
                },
            };

            let file_metadata = match last_modified(file) {
                Ok(m) => m.max(1),  // let's avoid 0 -> see the Err(e) branch
                Err(e) => {
                    // TODO: warn the programmer that incremental compilation is not working
                    // compiler_output.push_warning(...);

                    0
                },
            };

            if let Err(mut e) = hir_session.save_to_file(&tmp_path, Some(file_metadata)) {
                compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
            }
        }

        if compiler_option.dump_hir {
            let res = hir_session.dump_hir();

            if let Some(path) = &compiler_option.dump_hir_to {
                if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                    compiler_output.push_error(e.set_context(FileErrorContext::DumpingHirToFile).to_owned().into());
                }
            }

            else {
                compiler_output.dump_to_stdout(res);
            }
        }

        hir_session.errors.clear();
        hir_session.warnings.clear();

        (Some(hir_session), compiler_output)
    }
}

pub fn generate_path_for_ir(
    original_file: &Path,
    ext: &str,
    create_path_if_not_exist: bool,
) -> Result<Path, FileError> {
    let file_name = file_name(original_file)?;

    let base_path = join(
        &parent(original_file)?,
        SAVE_IRS_AT,
    )?;

    if exists(&base_path) {
        if !is_dir(&base_path) && create_path_if_not_exist {
            return Err(FileError::cannot_create_file(
                false, // there exists a file, not dir
                &base_path,
            ));
        }
    }

    else if create_path_if_not_exist {
        create_dir(&base_path)?;
    }

    let save_ir_to = join(
        &base_path,

        // TODO: how about using `set_ext`?
        &format!("{file_name}.{ext}"),
    )?;

    Ok(save_ir_to)
}

/// None: cannot find saved_ir\
/// Some(Err(e)): found saved_ir, but got an error while constructing the session from saved_ir\
/// Some(Ok(s)): successfully reconstructed the session from saved_ir
pub fn try_construct_session_from_saved_ir<T: Endec>(file: &Path, ext: &str) -> Option<Result<T, EndecError>> {
    let path_for_ir = if let Ok(path) = generate_path_for_ir(file, ext, false) {
        path
    } else {
        return None;
    };

    let file_metadata = match last_modified(file) {
        Ok(m) => m,
        Err(e) => {
            return Some(Err(e.into()));
        },
    };

    if exists(&path_for_ir) {
        match T::load_from_file(&path_for_ir, Some(file_metadata)) {
            // ir is older than the code -> the session has to be constructed from scratch!
            Err(EndecError {
                kind: EndecErrorKind::FileIsModified,
                ..
            }) => None,
            res => Some(res),
        }
    }

    else {
        None
    }
}

fn is_human_readable(file: &Path) -> bool {
    if let Ok(buf) = read_bytes(file) {
        if let Ok(s) = String::from_utf8(buf) {
            for c in s.chars() {
                let c = c as u32;

                // non-readable characters
                if c < 9 || 12 < c && c < 32 {
                    return false;
                }
            }

            return true;
        }
    }

    false
}
