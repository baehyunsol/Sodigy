use super::ParseErrorKind;
use crate::parse_file;
use crate::pattern::is_eq_pat_err;
use crate::session::LocalParseSession;
use crate::utils::bytes_to_string;
use sdg_fs::{read_string, write_bytes, WriteMode};

pub fn is_eq(k1: &ParseErrorKind, k2: &ParseErrorKind) -> bool {

    match k1 {
        ParseErrorKind::UnexpectedChar(c1) => match k2 {
            ParseErrorKind::UnexpectedChar(c2) if c1 == c2 => true,
            _ => false,
        },
        ParseErrorKind::UnexpectedEof => match k2 {
            ParseErrorKind::UnexpectedEof => true,
            _ => false,
        },
        ParseErrorKind::UnexpectedEoe(e1) => match k2 {
            ParseErrorKind::UnexpectedEoe(e2) => e1.is_same_type(e2),
            _ => false,
        },
        ParseErrorKind::UnexpectedToken { expected: e1, got: t1 } => match k2 {
            ParseErrorKind::UnexpectedToken { expected: e2, got: t2 } => e1.is_same_type(e2) && t1.is_same_type(t2),
            _ => false,
        },
        ParseErrorKind::InvalidUTF8(e1) => match k2 {
            ParseErrorKind::InvalidUTF8(e2) => e1 == e2,
            _ => false,
        },

        ParseErrorKind::FileError(e1) => match k2 {
            ParseErrorKind::FileError(e2) => e1 == e2,
            _ => false,
        }

        // the test runner cannot generate an InternedString before it actually parses a code
        ParseErrorKind::UntypedArg(_, _) => match k2 {
            ParseErrorKind::UntypedArg(_, _) => true,
            _ => false,
        },
        ParseErrorKind::MultipleDefParam(_, t1) => match k2 {
            ParseErrorKind::MultipleDefParam(_, t2) => t1 == t2,
            _ => false,
        },
        ParseErrorKind::InvalidPattern(p1) => match k2 {
            ParseErrorKind::InvalidPattern(p2) => is_eq_pat_err(p1, p2),
            _ => false,
        }
    }

}

#[test]
fn error_message_test() {
    let mut session = LocalParseSession::new();
    let samples = (0..4096).map(|i| i.to_string()).collect::<Vec<String>>();
    let samples = vec![
        vec![
            "no_utf8_str".to_string(),
            "no_utf8_ident".to_string(),
            "no_utf8_comment".to_string(),
        ],
        samples,
    ].concat();

    // def DUMB_STRING: String = '������';
    let no_utf8_str = vec![100, 101, 102, 32, 68, 85, 77, 66, 95, 83, 84, 82, 73, 78, 71, 58, 32, 83, 116, 114, 105, 110, 103, 32, 61, 32, 39, 200, 200, 200, 200, 200, 200, 39, 59];
    write_bytes("./src/err/samples/no_utf8_str.in", &no_utf8_str, WriteMode::CreateOrTruncate).unwrap();

    // def ���: String = 'ABC';
    let no_utf8_ident = vec![100, 101, 102, 32, 200, 200, 200, 58, 32, 83, 116, 114, 105, 110, 103, 32, 61, 32, 39, 65, 66, 67, 39, 59];
    write_bytes("./src/err/samples/no_utf8_ident.in", &no_utf8_ident, WriteMode::CreateOrTruncate).unwrap();

    // invalid utf-8 in a comment
    let no_utf8_comment = [35, 32, 200, 200, 200, 10, 100, 101, 102, 32, 70, 79, 79, 58, 32, 73, 110, 116, 32, 61, 32, 48, 59];
    write_bytes("./src/err/samples/no_utf8_comment.in", &no_utf8_comment, WriteMode::CreateOrTruncate).unwrap();

    let mut failures = vec![];

    for sample in samples.iter() {
        if session.set_input(&format!("./src/err/samples/{sample}.in")).is_err() {
            break;
        }

        let input = session.get_curr_file_content().to_vec();

        let error_msg = if let Ok(s) = read_string(&format!("./src/err/samples/{sample}.out")) { s } else {
            format!("`{sample}.out` is not found!")
        };

        if let Err(_) = parse_file(&input, &mut session) {
            // let's not care about whitespaces
            let rendered_no_whitespace = session.render_err().chars().filter(|c| *c != ' ').collect::<String>();
            let msg_no_whitespace = error_msg.chars().filter(|c| *c != ' ').collect::<String>();

            if rendered_no_whitespace != msg_no_whitespace {
                failures.push(format!("\n\n---\n\nexpected\n{}\n\nactual\n{}", error_msg, session.render_err()));
            }
        } else {
            failures.push(format!("\n\n---\n\n{} is supposed to fail, but doesn't!", bytes_to_string(&input)));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.concat());
    }
}
