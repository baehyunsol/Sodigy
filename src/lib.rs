#![deny(unused_imports)]

use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_err::SodigyError;
use sodigy_files::{global_file_session, FileError, FileHash};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_interpreter::{HirEvalCtxt, eval_hir};
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

mod result;

#[cfg(test)]
mod tests;

use result::CompileResult;

pub fn compile_file(path: String) -> Result<CompileResult, FileError> {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_file(&path)?;

    Ok(compile(file))
}

pub fn compile_input(input: Vec<u8>) -> CompileResult {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_tmp_file(input);

    compile(file)
}

pub fn compile(file_hash: FileHash) -> CompileResult {
    let mut result = CompileResult::new();
    let file_session = unsafe { global_file_session() };
    let input = file_session.get_file_content(file_hash).unwrap();

    let mut lex_session = LexSession::new();

    if let Err(()) = lex(input, 0, SpanPoint::at_file(file_hash, 0), &mut lex_session) {
        for error in lex_session.get_errors() {
            result.push_error(error.to_universal());
        }

        return result;
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_tokens();
    let mut new_lex_session = LexSession::new();

    if let Err(()) = from_tokens(tokens, &mut parse_session, &mut new_lex_session) {
        for error in parse_session.get_errors() {
            result.push_error(error.to_universal());
        }

        for error in new_lex_session.get_errors() {
            result.push_error(error.to_universal());
        }

        return result;
    };

    for warning in parse_session.get_warnings() {
        result.push_warning(warning.to_universal());
    }

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    let mut tokens = parse_session.get_tokens().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);
    let res = parse_stmts(&mut tokens, &mut ast_session);

    for warning in ast_session.get_warnings() {
        result.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in ast_session.get_errors() {
            result.push_error(error.to_universal());
        }

        return result;
    }

    let mut hir_session = HirSession::new();
    let res = lower_stmts(ast_session.get_stmts(), &mut hir_session);

    for warning in hir_session.get_warnings() {
        result.push_warning(warning.to_universal());
    }

    if let Err(()) = res {
        for error in hir_session.get_errors() {
            result.push_error(error.to_universal());
        }

        return result;
    }

    // TODO: this is a tmp code for testing
    {
        let main = sodigy_intern::intern_string(b"main".to_vec());

        if let Some(main_func) = hir_session.func_defs.get(&main) {
            let main_func = main_func.clone();

            let mut eval_ctxt = HirEvalCtxt::from_session(&hir_session);

            match eval_hir(&main_func.ret_val, &mut eval_ctxt) {
                Ok(v) => {
                    println!("result: {v}");
                },
                Err(e) => {
                    println!("result: eval_hir failed: {e:?}");
                },
            }
        }

        else {
            println!("result: no main function");
        }
    }

    result
}
