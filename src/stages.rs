use crate::{CompilerOutput, SAVE_IRS_AT};
use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_endec::{Endec, EndecError, EndecErrorContext, EndecErrorKind};
use sodigy_error::UniversalError;
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
use sodigy_intern::InternedString;
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_session::SodigySession;
use sodigy_span::SpanPoint;
use std::collections::HashMap;

type Path = String;

#[derive(Clone, Copy)]
pub enum PathOrRawInput<'a> {
    Path(&'a Path),
    RawInput(&'a Vec<u8>),
}

const FILE_EXT_TOKENS: &str = "tokens";
const FILE_EXT_HIGH_IR: &str = "hir";

// TODO: this file has a lot of duplicate code blocks
// TODO: nicer return type for all the stages

pub fn parse_file(
    input: PathOrRawInput,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<ParseSession>, CompilerOutput) {
    let mut compiler_output = prev_output.unwrap_or_default();
    let file_session = unsafe { global_file_session() };

    let file_hash = match input {
        PathOrRawInput::Path(file) => {
            if let Some(s) = try_construct_session_from_saved_ir::<ParseSession>(file, FILE_EXT_TOKENS) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {},
                    Ok(session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&session);

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
                        compiler_output.push_warning(incremental_compilation_broken(file, e.into()));
                    },
                }
            }

            match file_session.register_file(file) {
                Ok(f) => f,
                Err(e) => {
                    compiler_output.push_error(e.into());
                    return (None, compiler_output);
                },
            }
        },
        PathOrRawInput::RawInput(raw_input) => match file_session.register_tmp_file(raw_input) {
            Ok(f) => {
                file_session.set_name_alias(f, "raw_input".to_string());

                f
            },
            Err(e) => {
                compiler_output.push_error(e.into());
                return (None, compiler_output);
            },
        },
    };

    let code = match file_session.get_file_content(file_hash) {
        Ok(f) => f,
        Err(e) => {
            compiler_output.push_error(e.into());

            return (None, compiler_output);
        },
    };

    let mut lex_session = LexSession::new();

    if let Err(()) = lex(code, 0, SpanPoint::at_file(file_hash, 0), &mut lex_session) {
        compiler_output.collect_errors_and_warnings_from_session(&lex_session);

        return (None, compiler_output);
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_results();
    let mut new_lex_session = LexSession::new();

    let mut res = from_tokens(tokens, &mut parse_session, &mut new_lex_session);

    if !parse_session.unexpanded_macros.is_empty() {
        let mut macro_definitions = HashMap::with_capacity(parse_session.unexpanded_macros.len());

        for macro_ in parse_session.unexpanded_macros.iter() {
            match try_get_macro_definition(*macro_) {
                Ok(m) => {
                    macro_definitions.insert(*macro_, m);
                },
                Err(e) => {
                    compiler_output.push_error(e);
                },
            }
        }

        res = parse_session.expand_macros(&macro_definitions);
    }

    compiler_output.collect_errors_and_warnings_from_session(&new_lex_session);
    compiler_output.collect_errors_and_warnings_from_session(&parse_session);

    if res.is_err() {
        (None, compiler_output)
    }

    else {
        match input {
            PathOrRawInput::Path(file) if compiler_option.save_ir => {
                if compiler_option.save_ir {
                    let tmp_path = match generate_path_for_ir(file, FILE_EXT_TOKENS, true) {
                        Ok(p) => p.to_string(),
                        Err(e) => {
                            compiler_output.push_error(e.into());
                            return (None, compiler_output);
                        },
                    };

                    let file_metadata = match last_modified(file) {
                        Ok(m) => m.max(1),  // let's avoid 0 -> see the Err(e) branch
                        Err(e) => {
                            compiler_output.push_warning(incremental_compilation_broken(file, e.into()));

                            0
                        },
                    };

                    if let Err(mut e) = parse_session.save_to_file(&tmp_path, Some(file_metadata)) {
                        compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
                    }
                }
            },
            _ => {},
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

        parse_session.clear_errors();
        parse_session.clear_warnings();

        (Some(parse_session), compiler_output)
    }
}

pub fn hir_from_tokens(
    input: PathOrRawInput,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<HirSession>, CompilerOutput) {
    let mut compiler_output = prev_output.unwrap_or_default();

    let parse_session = match input {
        PathOrRawInput::Path(file) => {
            if let Some(s) = try_construct_session_from_saved_ir::<HirSession>(file, FILE_EXT_HIGH_IR) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {},
                    Ok(session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&session);

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
                        compiler_output.push_warning(incremental_compilation_broken(file, e.into()));
                    },
                }
            }

            match IrStage::try_infer_from_ext(file) {

                // This file contains ParseSession
                Some(IrStage::Tokens) => match ParseSession::load_from_file(file, None) {
                    Ok(parse_session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&parse_session);

                        // We don't allow an erroneous session to continue compilation
                        if parse_session.has_error() {
                            return (None, compiler_output);
                        }

                        parse_session
                    },
                    Err(e) => {
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
                        compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                        return (Some(hir_session), compiler_output);
                    },
                    Err(e) => {
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
                    PathOrRawInput::Path(file),
                    None,
                    compiler_option,
                ) {
                    (Some(parse_session), output) => {
                        compiler_output.merge(output);

                        parse_session
                    },
                    (None, output) => {
                        return (None, output);
                    },
                },
            }
        },
        _ => {
            let (parse_session, compiler_output_) = parse_file(
                input,
                Some(compiler_output),
                compiler_option,
            );

            compiler_output = compiler_output_;

            match parse_session {
                Some(parse_session) => parse_session,
                None => {
                    return (None, compiler_output);
                },
            }
        },
    };

    // It's an internal compiler error
    // macros are either
    // 1. all expanded at the parse stage
    // 2. make parse_session invalid so that the control flow can never reach here
    if !parse_session.unexpanded_macros.is_empty() {
        unreachable!();
    }

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    let mut tokens = parse_session.get_results().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);
    let res = parse_stmts(&mut tokens, &mut ast_session);

    compiler_output.collect_errors_and_warnings_from_session(&ast_session);

    if res.is_err() {
        return (None, compiler_output);
    }

    let mut hir_session = HirSession::new();
    let res = lower_stmts(ast_session.get_results(), &mut hir_session);

    compiler_output.collect_errors_and_warnings_from_session(&hir_session);

    if res.is_err() {
        (None, compiler_output)
    }

    else {
        match input {
            PathOrRawInput::Path(file) if compiler_option.save_ir => {
                let tmp_path = match generate_path_for_ir(file, FILE_EXT_HIGH_IR, true) {
                    Ok(p) => p.to_string(),
                    Err(e) => {
                        compiler_output.push_error(e.into());
                        return (None, compiler_output);
                    },
                };

                let file_metadata = match last_modified(file) {
                    Ok(m) => m.max(1),  // let's avoid 0 -> see the Err(e) branch
                    Err(e) => {
                        compiler_output.push_warning(incremental_compilation_broken(file, e.into()));

                        0
                    },
                };

                if let Err(mut e) = hir_session.save_to_file(&tmp_path, Some(file_metadata)) {
                    compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
                }
            },
            _ => {},
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

        hir_session.clear_errors();
        hir_session.clear_warnings();

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
        Err(e) if e.is_file_not_found_error() => {
            return None;
        },
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

// it has to get the definitions from another file
// for `a.sdg` to use a macro named `foo`, two things are needed
// 1. a file that tells `a.sdg` where `foo` is defined
// 2. actual definition of `foo`
// 2 is an independent sodigy file. the compiler compiles the file and runs the macro
// using its interpreter. the compiler has to check whether 2 and `a.sdg` are depending
// on each other. otherwise, the compilation would stuck
// 1 is not a sodigy file. Since macros are applied before the name resolution,
// one cannot 'import' macros. there must be some kinda meta file for dependencies,
// like `Cargo.toml` for Rust and `go.mod` for Go.
fn try_get_macro_definition(name: InternedString) -> Result<(), UniversalError> {
    todo!()
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

fn incremental_compilation_broken(file: &Path, mut error: UniversalError) -> UniversalError {
    error.is_warning = true;
    error.append_message(&format!(
        "Incremental compilation on `{file}` is not working due to this error.\nIf you haven't messed up with `__sdg_cache__` directoy, this must be an internal compiler error. Please report this bug."
    ));

    error
}
