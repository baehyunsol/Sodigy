use super::ParseErrorKind;
use crate::parse_file;
use crate::file_system::{read_bytes, read_string};
use crate::session::LocalParseSession;

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
        }
    }

}

#[test]
fn error_message_test() {
    let mut session = LocalParseSession::new();

    for i in 0..3 {
        let input = read_bytes(&format!("./src/err/samples/{i}.in")).unwrap();
        session.set_input(input.clone());
        let error_msg = read_string(&format!("./src/err/samples/{i}.out")).unwrap();

        if let Err(e) = parse_file(&input, &mut session) {

            if e.render_err(&session) != error_msg {
                panic!("expected\n{}\n\nactual\n{}", error_msg, e.render_err(&session));
            }

        } else {
            panic!("{} is supposed to fail, but doesn't!", String::from_utf8_lossy(&input))
        }
    }
}
