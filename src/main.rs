use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_err::SodigyError;
use sodigy_files::{get_all_sdg, global_file_session, FileError, FileHash};
use sodigy_high_ir::{lower_stmts, HirSession};
use sodigy_lex::{lex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

mod result;

#[cfg(test)]
mod tests;

use result::CompileResult;

fn main() {
    // tests

    compile_file("./samples/easy.sdg".to_string()).unwrap().print_results();

    compile_input("
        def korean = \"한글 테스트 하나둘 하나둘\" <> \"셋넷\";
    ".as_bytes().to_vec()).print_results();

    for file in get_all_sdg(
        "./samples/err", false, "in"
    ).unwrap().iter().chain(
        get_all_sdg("./samples", true, "sdg").unwrap().iter()
    ) {
        compile_file(file.to_string()).unwrap().print_results();
    }
}

fn compile_file(path: String) -> Result<CompileResult, FileError> {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_file(&path)?;

    Ok(compile(file))
}

fn compile_input(input: Vec<u8>) -> CompileResult {
    let file_session = unsafe { global_file_session() };
    let file = file_session.register_tmp_file(input);

    compile(file)
}

fn compile(file_hash: FileHash) -> CompileResult {
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

    result
}
