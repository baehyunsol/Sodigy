use crate::{CompilerOutput, DEPENDENCIES_AT, SAVE_IRS_AT};
use crate::error;
use crate::global_hir_cache::{init_global_hir_cache, GlobalHirCache};
use crate::multi::{self, MessageFromMain, MessageToMain};
use log::info;
use sodigy_ast::{
    parse_config_file,
    parse_stmts,
    AstSession,
    IdentWithSpan,
    Tokens,
};
use sodigy_clap::{CompilerOption, Flag};
use sodigy_endec::{DumpJson, Endec, EndecError, EndecErrorKind};
use sodigy_error::{
    ErrorContext,
    RenderError,
    SodigyError,
    UniversalError,
};
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
use sodigy_span::{SpanPoint, SpanRange};
use std::collections::HashMap;
use std::sync::mpsc;

type Path = String;

#[derive(Clone, Copy, Debug)]
pub enum PathOrRawInput<'a> {
    Path(&'a Path),
    RawInput(&'a Vec<u8>),
}

// If `construct_XXX` returns `(Some(session), output)`, the `output` only contains errors that
// have nothing to do with sessions: eg) file errors
// otherwise, the `output` contains all the errors that it got during the compilation

// `construct_XXX().0.is_some()` doesn't mean that the compilation was successful.
// sometimes failed compilations return `Some(session)`.

const FILE_EXT_HIGH_IR: &str = "hir";
const FILE_EXT_MID_IR: &str = "mir";

pub fn construct_hir(
    input: PathOrRawInput,
    compiler_option: &CompilerOption,
) -> (Option<HirSession>, CompilerOutput) {
    info!("sodigy::construct_hir() with input: {input:?}");
    let mut compiler_output = CompilerOutput::new();
    let file_session = unsafe { global_file_session() };

    let file_hash = match input {
        PathOrRawInput::Path(file) => {
            // if HirSession is saved as a file and it's up to date, it just constructs the session from the file and returns
            if let Some(s) = try_construct_session_from_saved_ir::<HirSession>(file, FILE_EXT_HIGH_IR) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {
                        info!("found session from previous compilation, but the dependencies are not up to date: (file: {file}, ext: {FILE_EXT_HIGH_IR})");
                    },
                    Ok(session) => {
                        info!("found session from previous compilation, and the dependencies are up to date: (file: {file}, ext: {FILE_EXT_HIGH_IR})");
                        warn_ignored_dumps(&mut compiler_output, compiler_option, FILE_EXT_HIGH_IR);

                        if let Some(path) = &compiler_option.dump_hir_to {
                            let res = session.dump_json().pretty(4);

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

            // it doesn't check whether there's cached MIR or not

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
            PathOrRawInput::Path(p) if !compiler_option.parse_config_file => match parent(p) {
                Ok(p) => p,
                Err(e) => {
                    compiler_output.collect_errors_and_warnings_from_session(&new_lex_session);
                    compiler_output.collect_errors_and_warnings_from_session(&parse_session);
                    compiler_output.push_error(e.into());
                    return (None, compiler_output);
                },
            },
            _ => {
                compiler_output.collect_errors_and_warnings_from_session(&new_lex_session);
                compiler_output.collect_errors_and_warnings_from_session(&parse_session);

                for macro_span in parse_session.unexpanded_macros.values() {
                    compiler_output.push_error(no_macro_in_raw_input(*macro_span));
                }

                return (None, compiler_output);
            },
        };

        for macro_ in parse_session.unexpanded_macros.keys() {
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

    if res.is_err() {
        compiler_output.collect_errors_and_warnings_from_session(&new_lex_session);
        compiler_output.collect_errors_and_warnings_from_session(&parse_session);
        return (None, compiler_output);
    }

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    ast_session.merge_errors_and_warnings(&new_lex_session);

    if compiler_option.parse_config_file {
        match parse_config_file(parse_session.get_results(), &mut ast_session) {
            Ok(new_tokens) => {
                *parse_session.get_results_mut() = new_tokens;
            },
            Err(_) => {
                compiler_output.collect_errors_and_warnings_from_session(&ast_session);

                return (None, compiler_output);
            },
        }
    }

    let mut tokens = parse_session.get_results().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);
    let res = parse_stmts(&mut tokens, &mut ast_session);

    if res.is_err() {
        compiler_output.collect_errors_and_warnings_from_session(&ast_session);
        return (None, compiler_output);
    }

    let mut hir_session = HirSession::from_ast_session(&ast_session);
    let _ = lower_stmts(ast_session.get_results(), &mut hir_session);

    match input {
        // it saves ir of failed compilations
        // so that it can fail faster if the user tries to compile the same file again
        PathOrRawInput::Path(file) if compiler_option.save_ir => {
            let tmp_path = match generate_path_for_ir(file, FILE_EXT_HIGH_IR, true) {
                Ok(p) => p.to_string(),
                Err(e) => {
                    compiler_output.push_error(e.into());
                    return (Some(hir_session), compiler_output);
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
        let res = hir_session.dump_json().pretty(4);

        if path != "STDOUT" {
            if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                compiler_output.push_error(e.set_context(FileErrorContext::DumpingHirToFile).to_owned().into());
            }
        }

        else {
            compiler_output.dump_to_stdout(res);
        }
    }

    (Some(hir_session), compiler_output)
}

pub fn construct_mir(
    input: PathOrRawInput,
    compiler_option: &CompilerOption,
) -> (Option<MirSession>, CompilerOutput) {
    info!("sodigy::construct_mir() with input: {input:?}");

    let mut compiler_output = CompilerOutput::new();

    let hir_session = match input {
        PathOrRawInput::Path(file) => {
            // if MirSession is saved as a file and it's up to date, it just constructs the session from the file and returns
            if let Some(s) = try_construct_session_from_saved_ir::<MirSession>(file, FILE_EXT_MID_IR) {
                match s {
                    Ok(session) if !session.check_all_dependency_up_to_date() => {
                        info!("found session from previous compilation, but the dependencies are not up to date: (file: {file}, ext: {FILE_EXT_MID_IR})");
                    },
                    Ok(session) => {
                        info!("found session from previous compilation, and the dependencies are up to date: (file: {file}, ext: {FILE_EXT_MID_IR})");
                        compiler_output.collect_errors_and_warnings_from_session(&session);
                        warn_ignored_dumps(&mut compiler_output, compiler_option, FILE_EXT_MID_IR);

                        if let Some(path) = &compiler_option.dump_mir_to {
                            let res = session.dump_json().pretty(4);

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

            match construct_hir(
                input,
                compiler_option,
            ) {
                (Some(hir_session), output) => {
                    compiler_output.merge(output);
                    hir_session
                },
                (None, output) => {
                    return (None, output);
                },
            }
        },
        _ => {
            match construct_hir(
                input,
                compiler_option,
            ) {
                (Some(hir_session), output) => {
                    compiler_output.merge(output);
                    hir_session
                },
                (None, output) => {
                    return (None, output);
                },
            }
        },
    };

    if hir_session.has_error() {
        compiler_output.collect_errors_and_warnings_from_session(&hir_session);
        return (None, compiler_output);
    }

    // where to look for other files
    let base_path = match &input {
        PathOrRawInput::Path(p) => match parent(p) {
            Ok(p) => p,
            Err(e) => {
                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                compiler_output.push_error(e.into());
                return (None, compiler_output);
            },
        },
        PathOrRawInput::RawInput(_) => String::from("."),
    };

    // check out https://github.com/baehyunsol/Sodigy/blob/c38f4fab18525da89fa20e0bd2a1c3ab938f6bb5/src/stages.rs#L354 for the previous implementation

    // it will later be merged to MIR Session
    let mut mir_session_dependencies = vec![];
    let global_hir_cache = unsafe { init_global_hir_cache() };

    if !hir_session.imported_names.is_empty() && matches!(&input, PathOrRawInput::RawInput(_)) {
        compiler_output.collect_errors_and_warnings_from_session(&hir_session);
        compiler_output.push_error(no_dependency_in_raw_input(
            *hir_session.imported_names[0].span()
        ));
        return (None, compiler_output);
    }

    if let Err(errors) = resolve_and_push_dependencies_from_hir_session(
        &hir_session.imported_names,
        &base_path,
        compiler_option,
        &mut mir_session_dependencies,
        global_hir_cache,
    ) {
        compiler_output.collect_errors_and_warnings_from_session(&hir_session);

        for error in errors.into_iter() {
            compiler_output.push_error(error);
        }

        return (None, compiler_output);
    }

    let hir_workers = multi::init_hir_workers(
        compiler_option.num_workers,
        compiler_option.clone(),
    );
    let mut worker_index = 0;

    while let Some((name, path)) = global_hir_cache.pop_job_queue() {
        if let Err(_) = hir_workers[worker_index % hir_workers.len()].send(
            MessageFromMain::ConstructHirSession { name, path }
        ) {
            multi::kill_all_workers(&hir_workers);
            compiler_output.collect_errors_and_warnings_from_session(&hir_session);
            compiler_output.push_error(mpsc_broken());

            return (None, compiler_output);
        }

        worker_index += 1;
    }

    if worker_index < hir_workers.len() {
        for index in worker_index..hir_workers.len() {
            if let Err(_) = hir_workers[index].send(MessageFromMain::YouShouldAskForAJob) {
                multi::kill_all_workers(&hir_workers);
                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                compiler_output.push_error(mpsc_broken());

                return (None, compiler_output);
            }
        }
    }

    loop {
        let mut idle_workers = 0;

        for worker in hir_workers.iter() {
            match worker.try_recv() {
                Ok(msg) => match msg {
                    MessageToMain::HirComplete { imported_names } => {
                        if let Err(errors) = resolve_and_push_dependencies_from_hir_session(
                            &imported_names,
                            &base_path,
                            compiler_option,
                            &mut mir_session_dependencies,
                            global_hir_cache,
                        ) {
                            compiler_output.collect_errors_and_warnings_from_session(&hir_session);

                            for error in errors.into_iter() {
                                compiler_output.push_error(error);
                            }

                            return (None, compiler_output);
                        }

                        if let Some((name, path)) = global_hir_cache.pop_job_queue() {
                            if let Err(_) = worker.send(MessageFromMain::ConstructHirSession { name, path }) {
                                multi::kill_all_workers(&hir_workers);
                                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                                compiler_output.push_error(mpsc_broken());

                                return (None, compiler_output);
                            }
                        }

                        else {
                            if let Err(_) = worker.send(MessageFromMain::YouShouldAskForAJob) {
                                multi::kill_all_workers(&hir_workers);
                                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                                compiler_output.push_error(mpsc_broken());

                                return (None, compiler_output);
                            }
                        }
                    },
                    MessageToMain::GiveMeAJob => {
                        if let Some((name, path)) = global_hir_cache.pop_job_queue() {
                            if let Err(_) = worker.send(MessageFromMain::ConstructHirSession { name, path }) {
                                multi::kill_all_workers(&hir_workers);
                                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                                compiler_output.push_error(mpsc_broken());

                                return (None, compiler_output);
                            }
                        }

                        else {
                            idle_workers += 1;

                            if let Err(_) = worker.send(MessageFromMain::YouShouldAskForAJob) {
                                multi::kill_all_workers(&hir_workers);
                                compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                                compiler_output.push_error(mpsc_broken());

                                return (None, compiler_output);
                            }
                        }
                    },
                },
                Err(e) => match e {
                    mpsc::TryRecvError::Empty => {},
                    mpsc::TryRecvError::Disconnected => {
                        multi::kill_all_workers(&hir_workers);
                        compiler_output.collect_errors_and_warnings_from_session(&hir_session);
                        compiler_output.push_error(mpsc_broken());

                        return (None, compiler_output);
                    },
                },
            }
        }

        if idle_workers == hir_workers.len() {
            multi::kill_all_workers(&hir_workers);
            break;
        }
    }

    if global_hir_cache.has_error() {
        compiler_output.collect_errors_and_warnings_from_session(&hir_session);

        for error in global_hir_cache.collect_all_errors_and_warnings() {
            // TODO: it collected errors and warnings, but why is it `push_error`?
            compiler_output.push_error(error);
        }

        return (None, compiler_output);
    }

    // TODO
    // Now that all the HIR Sessions are complete,
    // 1, collect names
    // 2, construct the MIR session
    // 3, free global_hir_cache

    todo!()
}

pub fn construct_binary(
    input: PathOrRawInput,
    compiler_option: &CompilerOption,
) -> (Option<Vec<u8>>, CompilerOutput) {
    info!("sodigy::construct_binary() with input: {input:?}");
    // TODO: binary stage is not implemented yet -> it only collects
    // errors and warnings until mir stage

    let (session, mut output) = construct_mir(input, compiler_option);

    if let Some(session) = session {
        output.collect_errors_and_warnings_from_session(&session);
    }

    (Some(vec![]), output)
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

// it doesn't look for the config file if it's compiling a raw input
fn try_get_macro_definition(base_path: &Path, name: InternedString) -> Result<(), UniversalError> {
    // reads the contents of `./sodigy.json`
    let dependencies = read_string(&join(base_path, DEPENDENCIES_AT)?)?;

    // what then?

    // TODO: more idea on config files
    // 1. now that the compiler can interpret config files,
    // 2. interpret config files and store them at a global cache

    todo!()
}

// Even though it returns Ok(path), you have to check whether `path` exists
// Make sure that it's not compiling a raw input. A raw input cannot have a dependency.
fn try_resolve_dependency(base_path: &Path, compiler_option: &CompilerOption, dependency: IdentWithSpan) -> Result<Path, UniversalError> {
    // see README: it tells you where to look for the dependencies
    let dep_name = dependency.id().to_string();

    // 1. check if CompilerOption knows where the file is
    // TODO: users cannot provide this option (not yet)
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

    // 3. check `sodigy.json`
    // TODO: not implemented yet

    // 4. std lib
    // TODO

    Err(error::dependency_not_found(dependency))
}

fn resolve_and_push_dependencies_from_hir_session(
    imported_names: &[IdentWithSpan],
    base_path: &Path,
    compiler_option: &CompilerOption,
    mir_session_dependencies: &mut Vec<SessionDependency>,
    global_hir_cache: &mut GlobalHirCache,
) -> Result<(), Vec<UniversalError>> {
    let mut errors = vec![];

    for name in imported_names.iter() {
        match try_resolve_dependency(base_path, compiler_option, *name) {
            Ok(path) => {
                if !is_file(&path) {
                    errors.push(MirError::file_not_found(*name, path.clone()).to_universal());
                    continue;
                }

                let last_modified_at = match last_modified(&path) {
                    Ok(m) => m,
                    Err(e) => {
                        let mut e = UniversalError::from(e);
                        e.push_span(*name.span());
                        errors.push(e);
                        continue;
                    },
                };

                mir_session_dependencies.push(SessionDependency {
                    path: path.clone(),
                    last_modified_at,
                });

                global_hir_cache.push_job_queue(name.id().to_string(), path);
            },
            Err(e) => {
                errors.push(e);
            },
        }
    }

    if errors.is_empty() {
        Ok(())
    }

    else {
        Err(errors)
    }
}

fn incremental_compilation_broken(file: &Path, mut error: UniversalError) -> UniversalError {
    error.is_warning = true;
    error.append_message(&format!(
        "Incremental compilation on `{file}` is not working due to this error.\nIf you haven't messed up with `__sdg_cache__` directoy, this must be an internal compiler error. Please report this bug."
    ));

    error
}

fn warn_ignored_dumps(output: &mut CompilerOutput, options: &CompilerOption, cached_ext: &str) {
    if cached_ext == FILE_EXT_HIGH_IR {
        // nothing is ignored
    }

    else if cached_ext == FILE_EXT_MID_IR {
        if let Some(path) = &options.dump_hir_to {
            output.push_warning(ignored_dump_warning(path, Flag::DumpHirTo, cached_ext));
        }
    }

    else {
        // no other stages yet
        unreachable!();
    }
}

fn ignored_dump_warning(path: &Path, flag: Flag, cached_ext: &str) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        true,   // is_warning
        false,  // show_span
        None,   // span
        format!(
            "`{}` ignored due to incremental compilation",
            flag.render_error(),
        ),
        format!("Since it's reading cached data of `{cached_ext}`, it writes nothing to `{path}`.\nIf you want to dump something, run `sodigy --clean` and try again."),
    )
}

fn no_macro_in_raw_input(span: SpanRange) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        false,  // is_warning
        true,   // show_span
        Some(span),  // span
        String::from("macros not allowed in raw input"),
        String::new(),
    )
}

fn no_dependency_in_raw_input(span: SpanRange) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        false,  // is_warning
        true,   // show_span
        Some(span),  // span
        String::from("dependencies not allowed in raw input"),
        String::new(),
    )
}

fn mpsc_broken() -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        false,  // is_warning
        false,  // show_span
        None,   // span
        String::from("broken mpsc channel"),

        // for now, users cannot disable mpsc
        String::new(),
    )
}
