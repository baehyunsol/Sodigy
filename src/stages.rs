use crate::CompilerOutput;
use log::{debug, info};
use sodigy_ast::{
    AstSession,
    Tokens,
    parse_stmts,
};
use sodigy_config::{CompilerOption, DumpType};
use sodigy_endec::DumpJson;
use sodigy_error::{
    UniversalError,
};
use sodigy_files::{
    FileErrorContext,
    WriteMode,
    global_file_session,
    write_string,
};
use sodigy_high_ir::{
    HirSession,
    lower_stmts,
};
use sodigy_intern::InternedString;
use sodigy_lex::{LexSession, lex};
use sodigy_mid_ir::{
    MirSession,
    lower_funcs,
};
use sodigy_parse::{ParseSession, from_tokens};
use sodigy_session::SodigySession;
use sodigy_span::SpanPoint;
use std::collections::HashMap;

type Path = String;

#[derive(Clone, Copy)]
pub enum PathOrRawInput<'a> {
    Path(&'a Path),
    RawInput(&'a Vec<u8>),
}

// If `construct_XXX` returns `(Some(session), output)`, the `output` only contains errors that
// have nothing to do with sessions: eg) file errors
// otherwise, the `output` contains all the errors that it got during the compilation

// `construct_XXX().0.is_some()` doesn't mean that the compilation was successful.
// sometimes failed compilations return `Some(session)`.

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

    let mut lex_session = LexSession::new(compiler_option.clone());

    if let Err(()) = lex(code, 0, SpanPoint::at_file(file_hash, 0), &mut lex_session) {
        info!("construct_hir({input:?}) failed at lex(...)");
        compiler_output.collect_errors_and_warnings_from_session(&lex_session);

        return (None, compiler_output);
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_results();
    let mut new_lex_session = LexSession::new(compiler_option.clone());

    let mut res = from_tokens(tokens, &mut parse_session, &mut new_lex_session);

    if !parse_session.unexpanded_macros.is_empty() {
        let mut macro_definitions = HashMap::with_capacity(parse_session.unexpanded_macros.len());

        for macro_ in parse_session.unexpanded_macros.keys() {
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

    if res.is_err() {
        info!("construct_hir({input:?}) failed at from_tokens(...)");
        compiler_output.collect_errors_and_warnings_from_session(&new_lex_session);
        compiler_output.collect_errors_and_warnings_from_session(&parse_session);
        return (None, compiler_output);
    }

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    ast_session.merge_errors_and_warnings(&new_lex_session);

    let mut tokens = parse_session.get_results().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);
    let res = parse_stmts(&mut tokens, &mut ast_session);

    if res.is_err() {
        info!("construct_hir({input:?}) failed at parse_stmts(...)");
        compiler_output.collect_errors_and_warnings_from_session(&ast_session);
        return (None, compiler_output);
    }

    let mut hir_session = HirSession::from_ast_session(&ast_session);
    let _ = lower_stmts(ast_session.get_results(), &mut hir_session);

    if let Some(path) = &compiler_option.dump_hir_to {
        let res = match compiler_option.dump_type {
            DumpType::Json => hir_session.dump_json().pretty(4),
            DumpType::String => hir_session.dump_hir(),
        };
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
) -> (Option<MirSession>, CompilerOutput) {
    info!("sodigy::construct_mir() with input: {input:?}");
    let (hir_session, mut compiler_output) = construct_hir(input, compiler_option);

    if hir_session.is_none() || compiler_output.has_error() {
        if let Some(hir_session) = &hir_session {
            compiler_output.collect_errors_and_warnings_from_session(hir_session);
        }

        return (None, compiler_output);
    }

    let hir_session = hir_session.unwrap();
    let mut mir_session = MirSession::from_hir_session(&hir_session);
    let _ = lower_funcs(&hir_session, &mut mir_session);

    if let Some(path) = &compiler_option.dump_mir_to {
        let res = match compiler_option.dump_type {
            DumpType::Json => mir_session.dump_json().pretty(4),
            DumpType::String => mir_session.dump_mir(),
        };
        debug!("dump_mir_to: {path:?}");

        if path != "STDOUT" {
            if let Err(mut e) = write_string(path, &res, WriteMode::CreateOrTruncate) {
                compiler_output.push_error(e.set_context(FileErrorContext::DumpingMirToFile).to_owned().into());
            }
        }

        else {
            compiler_output.dump_to_stdout(res);
        }
    }

    info!("construct_mir() for {:?} successfully completed", input);
    (Some(mir_session), compiler_output)
}

// it returns `()` because it's not implemented yet
fn try_get_macro_definition(name: InternedString) -> Result<(), UniversalError> {
    // TODO
    Ok(())
}
