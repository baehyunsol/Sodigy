use crate::{CompilerOutput, DEPENDENCIES_AT, SAVE_IRS_AT};
use crate::error;
use log::{debug, info};
use sodigy_ast::{
    AstSession,
    IdentWithSpan,
    Tokens,
    parse_config_file,
    parse_stmts,
};
use sodigy_clap::Flag;
use sodigy_config::CompilerOption;
use sodigy_endec::{DumpJson, Endec};
use sodigy_error::{
    ErrorContext,
    RenderError,
    UniversalError,
};
use sodigy_files::{
    FileError,
    FileErrorContext,
    WriteMode,
    create_dir,
    exists,
    file_name,
    global_file_session,
    is_dir,
    is_file,
    join,
    parent,
    read_string,
    set_extension,
    write_string,
};
use sodigy_high_ir::{
    HirSession,
    lower_stmts,
};
use sodigy_intern::InternedString;
use sodigy_lex::{LexSession, lex};
use sodigy_parse::{ParseSession, from_tokens};
use sodigy_session::SodigySession;
use sodigy_span::{SpanPoint, SpanRange};
use std::collections::HashMap;

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

    // TODO
    //      1. no more `save_ir` option
    //      2. save hir only when the final output is set to hir (`-H` flag)
    //      3. save the output to OUTPUT, not to a tmp path
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

            if let Err(mut e) = hir_session.save_to_file(&tmp_path) {
                compiler_output.push_error(e.set_context(FileErrorContext::SavingIr).to_owned().to_owned().into());
            }
        },

        // TODO: it has to save the hir of raw input
        //       there's no save_ir option anymore...
        _ => {},
    }

    if let Some(path) = &compiler_option.dump_hir_to {
        let res = hir_session.dump_json().pretty(4);
        debug!("dump_hir_to: {path:?}");

        if path != "STDOUT" {
            if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                compiler_output.push_error(e.set_context(FileErrorContext::DumpingHirToFile).to_owned().into());
            }
        }

        else {
            compiler_output.dump_to_stdout(res);
        }
    }

    info!("construct_hir() for {:?} successfully completed", input);
    (Some(hir_session), compiler_output)
}

pub fn construct_mir(
    input: PathOrRawInput,
    compiler_option: &CompilerOption,
) -> (
    Option<HirSession>,  // TODO: it has to be `MirSession`, but that's not implemented yet
    CompilerOutput,
) {
    info!("sodigy::construct_mir() with input: {input:?}");
    construct_hir(input, compiler_option)

    // TODO: construct mir from hir
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
