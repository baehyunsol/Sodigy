use crate::err::{ExpectedToken, ParseError, ParseErrorKind, SodigyError};
use crate::expr::{parse_expr, Expr};
use crate::lexer::lex_tokens;
use crate::session::LocalParseSession;
use crate::token::{OpToken, TokenKind, TokenList};
use hmath::Ratio;

pub fn dump_ast_of_expr(
    input: Vec<u8>,
    session: &mut LocalParseSession,
) -> Result<Expr, ParseError> {
    session.set_input(input.clone());

    let tokens = lex_tokens(&input, session)?;
    let mut token_list = TokenList::from_vec(tokens);
    let expr = parse_expr(&mut token_list, 0)?;

    assert!(token_list.is_eof());

    Ok(expr)
}

fn valid_samples() -> Vec<(Vec<u8>, String, usize)> {  // (input, AST, span of the top operator)
    let result = vec![
        ("a[1]", "Index(a,1)", 1),
        ("a.b.c(3)", "Call(Path(Path(a,b),c),3)", 5),
        (
            "-1 + -2 * -3 + -4",
            "Add(Add(Neg(1),Mul(Neg(2),Neg(3))),Neg(4))",
            13,
        ),
        ("-(a+b)*-(c+d)", "Mul(Neg(Add(a,b)),Neg(Add(c,d)))", 6),
        ("---3---2", "Sub(Neg(Neg(Neg(3))),Neg(Neg(2)))", 4),
        ("1*2/3%4", "Rem(Div(Mul(1,2),3),4)", 5),
        (
            "foo(3, 4 + 1, bar(5)) + 2",
            "Add(Call(foo,3,Add(4,1),Call(bar,5)),2)",
            22,
        ),
        ("a() + b()", "Add(Call(a),Call(b))", 4),
        (
            "a(1).b(2).c(3).d",
            "Path(Call(Path(Call(Path(Call(a,1),b),2),c),3),d)",
            14,
        ),
        (
            "-a().b() + -c().d()",
            "Add(Neg(Call(Path(Call(a),b))),Neg(Call(Path(Call(c),d))))",
            9,
        ),
        (
            "-a()[0] + -b()[1]",
            "Add(Neg(Index(Call(a),0)),Neg(Index(Call(b),1)))",
            8,
        ),
        (
            "[0][1][2][3].a",
            "Path(Index(Index(Index([0],1),2),3),a)",
            12,
        ),
        ("1.2 + a.b", "Add(1.2,Path(a,b))", 4),
        (
            "a[1..2] <> b[1..]",
            "Concat(Index(a,Range(1,2)),Index(b,Range(1)))",
            8,
        ),
        (
            "[1.2.., 1.2..3.4, 1. .. 3.]",
            "[Range(1.2),Range(1.2,3.4),Range(1,3)]",
            0,
        ),
        ("1. ..", "Range(1)", 3),
        ("1.0..", "Range(1)", 3),
        ("[[], [], ]", "[[],[]]", 0),
        ("1.", "1", 0),
        ("[\"Trailing Comma\", ]", "[\"Trailing Comma\"]", 0),
        ("[1, 2, 3, [4, 5, 6]]", "[1,2,3,[4,5,6]]", 0),
        (
            "x > y && y > 1 || x > z && z > 1",
            "LogicalOr(LogicalAnd(Gt(x,y),Gt(y,1)),LogicalAnd(Gt(x,z),Gt(z,1)))",
            15,
        ),
        ("(foo(1))(2)", "Call(Call(foo,1),2)", 8),
        ("{x = 3; y = x + 1; x + y}", "{x=3;y=Add(x,1);Add(x,y)}", 0),
        (
            "(3 > 4 != 5 < 6) == True",
            "Eq(Ne(Gt(3,4),Lt(5,6)),True)",
            17,
        ),
        (
            "if x > y { x } else { y } * 2",
            "Mul(Branch(Gt(x,y),x,y),2)",
            26,
        ),
        (
            "if x > y { x } else if x < y { y } else { 0 } * 2",
            "Mul(Branch(Gt(x,y),x,Branch(Lt(x,y),y,0)),2)",
            46,
        ),
        (
            "if if a { b } else { c } { d } else { e }",
            "Branch(Branch(a,b,c),d,e)",
            0,
        ),
        ("\\{x: Int, y: Int, x + y}", "Lambda(x:Int,y:Int,Add(x,y))", 0),
        ("(3)", "3", 1),
        ("(3,)", "Tuple(3)", 0),
        ("(3, 4)", "Tuple(3,4)", 0),
        ("(3, 4,)", "Tuple(3,4)", 0),
        ("()", "Tuple()", 0),
    ];

    result
        .into_iter()
        .map(|(input, ast, span)| (input.bytes().collect::<Vec<u8>>(), ast.to_string(), span))
        .collect()
}

fn invalid_samples() -> Vec<(Vec<u8>, ParseErrorKind, usize)> {  // (input, error kind, error span)
    let result = vec![
        ("1...3.", ParseErrorKind::UnexpectedChar('.'), 1),
        ("1...", ParseErrorKind::UnexpectedChar('.'), 1),
        (
            "a.1",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier()
                ]),
            },
            1,
        ),
        (
            "[1, 2, a[]]",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            8,
        ),
        ("[(), {), ]", ParseErrorKind::UnexpectedChar(')'), 6),
        ("[1, 2, 3, 4", ParseErrorKind::UnexpectedEof, 11),
        (
            "if x { 0 } else { }",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            16,
        ),
        (
            "if x > y { x } * 2",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Operator(OpToken::Mul),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::keyword_else(),
                ]),
            },
            15,
        ),
        (
            "if x > y { x }",
            ParseErrorKind::UnexpectedEoe(
                ExpectedToken::SpecificTokens(vec![
                    TokenKind::keyword_else(),
                ])
            ),
            0,
        ),
        (
            "{a = 3; b = 4;}",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            13,
        ),
        (
            "{1 1}",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::Nothing,
            },
            3,
        ),
        (
            "[1 1]",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::Nothing,
            },
            3,
        ),
        (
            "[1 1, 1 1]",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::Nothing,
            },
            3,
        ),
        (
            "(1 1)",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::Nothing,
            },
            3,
        ),
        (
            "foo(1 1)",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::Nothing,
            },
            6,
        ),
    ];

    result
        .into_iter()
        .map(|(input, err, span)| (input.bytes().collect::<Vec<u8>>(), err, span))
        .collect()
}

#[test]
fn valid_ast_dump_test() {
    let mut session = LocalParseSession::new();

    for (input, ast, span) in valid_samples() {
        println!("{}", String::from_utf8_lossy(&input));
        match dump_ast_of_expr(input, &mut session) {
            Ok(expr) => {
                assert_eq!(expr.to_string(&session), ast);
                assert_eq!(expr.span.index, span);
            }
            Err(err) => {
                panic!("{}", err.render_err(&session));
            }
        }
    }
}

#[test]
fn invalid_ast_dump_test() {
    let mut session = LocalParseSession::new();

    for (input, err_kind, span) in invalid_samples() {
        if let Err(e) = dump_ast_of_expr(input.clone(), &mut session) {
            if e.kind != err_kind {
                panic!(
                    "input: {}\nexpected_err:{}\ngot: {}",
                    String::from_utf8_lossy(&input).to_string(),
                    err_kind.render_err(&session),
                    e.kind.render_err(&session),
                );
            }

            assert_eq!(e.span.index, span);
        } else {
            panic!(
                "{:?} is supposed to fail, but doesn't!",
                String::from_utf8_lossy(&input).to_string()
            );
        }
    }
}
