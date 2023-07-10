use super::ParseErrorKind;
use crate::err::SodigyError;
use crate::expr::dump_ast_of_expr;
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
    let input = b"
[[
    1,
    2,
    3,
    4[],
    5
]]"
    .to_vec();

    session.set_input(input.clone());
    let error_msg = "Any kind of expression was expected, but got nothing!
    1 │ [[
    2 │     1,
    3 │     2,
    4 │     3,
      │      ▼
>>> 5 │     4[],
      │      ▲
    6 │     5
    7 │ ]]";

    if let Err(e) = dump_ast_of_expr(input, &mut session) {
        assert_eq!(e.render_err(&session), error_msg)
    } else {
        panic!()
    }
}
