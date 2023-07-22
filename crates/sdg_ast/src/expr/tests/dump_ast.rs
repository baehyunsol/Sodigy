use crate::err::{ExpectedToken, ParamType, ParseError, ParseErrorKind, SodigyError, tests::is_eq};
use crate::expr::{parse_expr, Expr};
use crate::lexer::lex_tokens;
use crate::session::{InternedString, LocalParseSession};
use crate::token::{Delimiter, OpToken, TokenKind, TokenList};
use crate::utils::bytes_to_string;
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
        ("a[1] # Comment Test", "Index(a,1)", 1),
        ("a[1] ##! Comment Test !##", "Index(a,1)", 1),
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
        ("\\{x, y, x + y}", "Lambda(x,y,Add(x,y))", 0),
        ("(3)", "3", 1),
        ("(3,)", "Tuple(3)", 0),
        ("(3, 4)", "Tuple(3,4)", 0),
        ("(3, 4,)", "Tuple(3,4)", 0),
        ("()", "Tuple()", 0),
        ("'한글 입력 테스트'", "\"한글 입력 테스트\"", 0),
        (
            "b\"ABC 한글 DEF\"",
            "Bytes(65,66,67,32,237,149,156,234,184,128,32,68,69,70)",
            0,
        ),
        (
            "b\'ABC 한글 DEF\'",
            "Bytes(65,66,67,32,237,149,156,234,184,128,32,68,69,70)",
            0,
        ),
        ("f\"{a} + {b} = {a + b}\"", "Format(a,\" + \",b,\" = \",Add(a,b))", 0),
        ("f\'{a} + {b} = {a + b}\'", "Format(a,\" + \",b,\" = \",Add(a,b))", 0),
        ("f\"{{{3}}}\"", "Format(3)", 0),
        ("f\"{3}\"", "Format(3)", 0),
        ("f\"{3} + {4}\"", "Format(3,\" + \",4)",0),
        ("f'A, B, {C}, D'", "Format(\"A, B, \",C,\", D\")", 0),
        ("f\"ABC\"", "\"ABC\"", 0),
        ("f\"\"", "\"\"", 0),
        ("b\"\"", "Bytes()", 0),
        ("me $age me.age + 1", "ModifyField(age)(me,Add(Path(me,age),1))", 3),
        (
            "{a: Int = 3; b: String = \"abc\"; a + b} # Yeah, it's a type error, but this test is not for that",
            "{a:Int=3;b:String=\"abc\";Add(a,b)}",
            0,
        ),
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
        ("1 + ", ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression), 2),
        (
            "a.1",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier()
                ]),
            },
            2,
        ),
        (
            "a.(a)",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::List(Delimiter::Parenthesis, vec![]),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::dummy_identifier()
                ]),
            },
            2,
        ),
        (
            "[1, 2, a[]]",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            8,
        ),
        ("[(), {), ]", ParseErrorKind::UnexpectedChar(')'), 6),
        (
            "[1, 2, 3, 4",
            ParseErrorKind::UnexpectedEoe(
                ExpectedToken::SpecificTokens(vec![TokenKind::Operator(OpToken::ClosingSquareBracket)])
            ),
            0
        ),
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
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingCurlyBrace),
                ]),
            },
            3,
        ),
        (
            "[1 1]",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingSquareBracket),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "[1 1, 1 1]",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingSquareBracket),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "a[1 1]",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingSquareBracket),
                ]),
            },
            4,
        ),
        (
            "(1 1)",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "foo(1 1)",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::one()),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            6,
        ),
        (
            "한글넣으면죽음?",
            ParseErrorKind::UnexpectedChar('한'),
            0,
        ),
        (
            "f'ABC {}'",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            6,
        ),
        (
            "f'ABC {1 + }'",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            9,
        ),
        (
            "f'ABC { [][]}'",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::AnyExpression),
            10,
        ),
        (
            "f'{1'",
            ParseErrorKind::UnexpectedEoe(ExpectedToken::SpecificTokens(vec![TokenKind::Operator(OpToken::ClosingCurlyBrace)])),
            2,
        ),
        (
            "(b \"ABC 한글 DEF\")",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::String(vec![]),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "(b \'ABC 한글 DEF\')",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::String(vec![]),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "(f \"{a} + {b} = {a + b}\")",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::String(vec![]),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "(f \'{a} + {b} = {a + b}\')",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::String(vec![]),
                expected: ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::ClosingParenthesis),
                    TokenKind::comma(),
                ]),
            },
            3,
        ),
        (
            "[0, 1, 2, 3] $0 1",
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Number(Ratio::zero()),
                expected: ExpectedToken::SpecificTokens(vec![TokenKind::dummy_identifier()]),
            },
            14,
        ),
        (
            "\\{x: Int, x: Int, x + x}",
            ParseErrorKind::MultipleDefParam(InternedString::dummy(), ParamType::LambdaParam),
            10,
        ),
        (
            "{x = 3; x = 4; x + x}",
            ParseErrorKind::MultipleDefParam(InternedString::dummy(), ParamType::BlockDef),
            8,
        ),
        (
            "   ##!##  # Unfinished Comment",
            ParseErrorKind::UnexpectedEof,
            3,
        ),
    ];

    let mut result: Vec<(Vec<u8>, ParseErrorKind, usize)> = result
        .into_iter()
        .map(|(input, err, span)| (input.bytes().collect::<Vec<u8>>(), err, span))
        .collect();

    result.push((
        vec![32, 32, 34, 65, 65, 200, 200, 65, 65, 34],
        ParseErrorKind::InvalidUTF8(vec![200]),
        5,
    ));

    result
}

#[test]
fn valid_ast_dump_test() {
    let mut session = LocalParseSession::new();

    for (input, ast, span) in valid_samples() {
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
            if !is_eq(&e.kind, &err_kind) || e.span.index != span {
                panic!(
                    "input: {}\nexpected_err:{}\ngot: {}",
                    bytes_to_string(&input),
                    err_kind.render_err(&session),
                    e.render_err(&session),
                );
            }
        } else {
            panic!(
                "{:?} is supposed to fail, but doesn't!",
                bytes_to_string(&input),
            );
        }
    }
}
