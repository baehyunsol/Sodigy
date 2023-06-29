use crate::expr::dump_ast_of_expr;
use crate::session::LocalParseSession;

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
]]".to_vec();
    session.set_input(input.clone());
    let error_msg = 
"Expected an expression, but got nothing!
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