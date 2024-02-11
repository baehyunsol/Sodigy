use crate::{CompilerOutput, DEPENDENCIES_AT, SAVE_IRS_AT};
use crate::error;
use log::info;
use sodigy_ast::{
    parse_stmts,
    AstSession,
    IdentWithSpan,
    Tokens,
};
use sodigy_clap::{CompilerOption, IrStage};
use sodigy_endec::{DumpJson, Endec, EndecError, EndecErrorContext, EndecErrorKind};
use sodigy_error::UniversalError;
use sodigy_files::{
    create_dir,
    exists,
    file_name,
    global_file_session,
    is_dir,
    is_file,
    join,
    last_modified,
    parent,
    read_bytes,
    read_string,
    set_extension,
    write_string,
    FileError,
    FileErrorContext,
    WriteMode,
};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_intern::InternedString;
use sodigy_lex::{lex, LexSession};
use sodigy_mid_ir::{MirError, MirSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_session::{SessionDependency, SodigySession};
use sodigy_span::SpanPoint;
use std::collections::{HashMap, HashSet};

type Path = String;

#[derive(Clone, Copy, Debug)]
pub enum PathOrRawInput<'a> {
    Path(&'a Path),
    RawInput(&'a Vec<u8>),
}

const FILE_EXT_HIGH_IR: &str = "hir";
const FILE_EXT_MID_IR: &str = "mir";

pub fn parse_file(
    input: PathOrRawInput,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<ParseSession>, CompilerOutput) {
    info!("sodigy::parse_file() with input: {input:?}");

    let mut compiler_output = prev_output.unwrap_or_default();
    let file_session = unsafe { global_file_session() };

    let file_hash = match input {
        PathOrRawInput::Path(file) => {
            // if ParseSession is saved as a file and it's up to date, it just constructs the session from the file and returns
            if let Some(s) = try_construct_session_from_saved_ir::<ParseSession>(file, FILE_EXT_TOKENS) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {},
                    Ok(session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&session);

                        if let Some(path) = &compiler_option.dump_tokens_to {
                            let res = session.dump_json().to_string();

                            if path != "STDOUT" {
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
        let base_path = match &input {
            PathOrRawInput::Path(p) => match parent(p) {
                Ok(p) => p,
                Err(e) => {
                    compiler_output.push_error(e.into());
                    return (None, compiler_output);
                },
            },
            PathOrRawInput::RawInput(_) => String::from("."),
        };

        for macro_ in parse_session.unexpanded_macros.iter() {
            match try_get_macro_definition(&base_path, *macro_) {
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

        if let Some(path) = &compiler_option.dump_tokens_to {
            let res = parse_session.dump_json().to_string();

            if path != "STDOUT" {
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
    info!("sodigy::hir_from_tokens() with input: {input:?}");

    let mut compiler_output = prev_output.unwrap_or_default();

    let parse_session = match input {
        PathOrRawInput::Path(file) => {
            // if HirSession is saved as a file and it's up to date, it just constructs the session from the file and returns
            if let Some(s) = try_construct_session_from_saved_ir::<HirSession>(file, FILE_EXT_HIGH_IR) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {
                        info!("found session from previous compilation, but the dependencies are not up to date: (file: {file}, ext: {FILE_EXT_HIGH_IR})");
                    },
                    Ok(session) => {
                        info!("found session from previous compilation, and the dependencies are up to date: (file: {file}, ext: {FILE_EXT_HIGH_IR})");
                        compiler_output.collect_errors_and_warnings_from_session(&session);

                        if let Some(path) = &compiler_option.dump_hir_to {
                            let res = session.dump_json().to_string();

                            if path != "STDOUT" {
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
                Some(IrStage::MidIr) => {
                    todo!()
                    // raise an error saying that,
                    // 'cannot downgrade an Mir to an Hir'
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

    let mut hir_session = HirSession::from_ast_session(&ast_session);
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

        if let Some(path) = &compiler_option.dump_hir_to {
            let res = hir_session.dump_json().to_string();

            if path != "STDOUT" {
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

pub fn mir_from_hir(
    input: PathOrRawInput,
    prev_output: Option<CompilerOutput>,
    compiler_option: &CompilerOption,
) -> (Option<MirSession>, CompilerOutput) {
    info!("sodigy::mir_from_hir() with input: {input:?}");

    let mut compiler_output = prev_output.unwrap_or_default();

    let hir_session = match input {
        PathOrRawInput::Path(file) => {
            // if MirSession is saved as a file and it's up to date, it just constructs the session from the file and returns
            if let Some(s) = try_construct_session_from_saved_ir::<MirSession>(file, FILE_EXT_MID_IR) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {},
                    Ok(session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&session);

                        if let Some(path) = &compiler_option.dump_mir_to {
                            let res = session.dump_json().to_string();

                            if path != "STDOUT" {  // TODO: use a constant
                                if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                                    compiler_output.push_error(e.set_context(FileErrorContext::DumpingMirToFile).to_owned().into());
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
                Some(IrStage::MidIr) => match MirSession::load_from_file(file, None) {  // MirSession is already here!
                    Ok(mir_session) => {
                        compiler_output.collect_errors_and_warnings_from_session(&mir_session);
                        return (Some(mir_session), compiler_output);
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

                _ => match hir_from_tokens(
                    PathOrRawInput::Path(file),
                    None,
                    compiler_option,
                ) {
                    (Some(hir_session), output) => {
                        compiler_output.merge(output);

                        hir_session
                    },
                    (None, output) => {
                        return (None, output);
                    },
                },
            }
        },
        _ => {
            let (hir_session, compiler_output_) = hir_from_tokens(
                input,
                Some(compiler_output),
                compiler_option,
            );

            compiler_output = compiler_output_;

            match hir_session {
                Some(hir_session) => hir_session,
                None => {
                    return (None, compiler_output);
                },
            }
        },
    };

    let base_path = match &input {
        PathOrRawInput::Path(p) => match parent(p) {
            Ok(p) => p,
            Err(e) => {
                compiler_output.push_error(e.into());
                return (None, compiler_output);
            },
        },
        PathOrRawInput::RawInput(_) => String::from("."),
    };

    let mut has_error = false;
    let mut mir_session = MirSession::from_hir_session(&hir_session);
    let mut construct_hirs_of_these = vec![];
    let mut paths_read_so_far = HashSet::new();

    if let PathOrRawInput::Path(p) = input {
        paths_read_so_far.insert(p.to_string());
    }

    let mut hir_sessions = vec![hir_session];

    while let Some(hir_session) = hir_sessions.pop() {
        if let Err(()) = mir_session.merge_hir(&hir_session) {
            has_error = true;
        }

        for name in hir_session.imported_names.iter() {
            match try_resolve_dependency(&base_path, compiler_option, *name) {
                Ok(path) => {
                    if paths_read_so_far.contains(&path) {
                        continue;
                    }

                    if !is_file(&path) {
                        has_error = true;
                        mir_session.push_error(MirError::file_not_found(*name, path.clone()));
                        continue;
                    }

                    let last_modified_at = match last_modified(&path) {
                        Ok(m) => m,
                        Err(e) => {
                            has_error = true;
                            let mut e = UniversalError::from(e);
                            e.push_span(*name.span());
                            compiler_output.push_error(e);

                            continue;
                        },
                    };

                    mir_session.add_dependency(SessionDependency {
                        path: path.clone(),
                        last_modified_at,
                    });

                    construct_hirs_of_these.push(path.clone());
                    paths_read_so_far.insert(path.clone());
                },
                Err(e) => {
                    has_error = true;
                    compiler_output.push_error(e);
                },
            }
        }

        // TODO
        // 1. construct hirs of files in `construct_hirs_of_these`
        // 2. i want it to run in parallel...
    }

    compiler_output.collect_errors_and_warnings_from_session(&mir_session);

    if has_error {
        return (None, compiler_output);
    }

    else {
        match input {
            PathOrRawInput::Path(file) if compiler_option.save_ir => {
                let tmp_path = match generate_path_for_ir(file, FILE_EXT_MID_IR, true) {
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

                if let Err(mut e) = mir_session.save_to_file(&tmp_path, Some(file_metadata)) {
                    compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
                }
            },
            _ => {},
        }

        if let Some(path) = &compiler_option.dump_mir_to {
            let res = mir_session.dump_json().to_string();

            if path != "STDOUT" {
                if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                    compiler_output.push_error(e.set_context(FileErrorContext::DumpingMirToFile).to_owned().into());
                }
            }

            else {
                compiler_output.dump_to_stdout(res);
            }
        }

        mir_session.clear_errors();
        mir_session.clear_warnings();

        (Some(mir_session), compiler_output)
    }
}

// for ex, `hir` (auto generated by compiler, not manually by the user) for `./foo.sdg` is at `./__sdg_cache__/foo.hir`
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
        &set_extension(&file_name, ext)?,
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

// TODO: what if it's compiling a raw input?
fn try_get_macro_definition(base_path: &Path, name: InternedString) -> Result<(), UniversalError> {
    // reads the contents of `./sodigy.toml`
    let dependencies = read_string(&join(base_path, DEPENDENCIES_AT)?)?;

    // what then?

    todo!()
}

// Even though it returns Ok(path), you have to check whether `path` exists
// TODO: it should behave differently when compiling raw inputs
fn try_resolve_dependency(base_path: &Path, compiler_option: &CompilerOption, dependency: IdentWithSpan) -> Result<Path, UniversalError> {
    // see README: it tells you where to look for the dependencies
    let dep_name = dependency.id().to_string();

    // 1. check if CompilerOption knows where the file is
    // TODO: users cannot provide this option
    if let Some(path) = compiler_option.dependencies.get(&dep_name) {
        return Ok(path.to_string());
    }

    // 2. check `./foo.sdg` and `./foo/lib.sdg`
    let candidate1 = join(base_path, &set_extension(&dep_name, "sdg")?)?;
    let candidate2 = join(
        base_path,
        &join(
            &dep_name,
            &set_extension("lib", "sdg")?,
        )?,
    )?;

    // `is_file` returns false if the path does not exist
    match (is_file(&candidate1), is_file(&candidate2)) {
        (true, true) => {
            return Err(error::conflicting_dependencies(
                dependency,
                candidate1,
                candidate2,
            ))
        },
        (true, false) => {
            return Ok(candidate1);
        },
        (false, true) => {
            return Ok(candidate2);
        },
        (false, false) => {
            // continue
        },
    }

    // 3. dependency file
    // TODO: not implemented yet

    // 4. std lib
    // TODO

    Err(error::dependency_not_found(dependency))
}

fn is_human_readable(file: &Path) -> bool {
    if let Ok(buffer) = read_bytes(file) {
        if let Ok(s) = String::from_utf8(buffer) {
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
