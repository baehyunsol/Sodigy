use crate::run;
use sodigy_config::CompilerOption;

macro_rules! check_output {
    (stmt, err, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_errors, $test_name, vec![], ($body).as_bytes().to_vec(), vec![], $msg);
    };
    (expr, err, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_errors, $test_name, b"let foo(x: Int, y: Int) = ".to_vec(), ($body).as_bytes().to_vec(), vec![b';'], $msg);
    };
    (stmt, warn, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_warnings, $test_name, vec![], ($body).as_bytes().to_vec(), vec![], $msg);
    };
    (expr, warn, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_warnings, $test_name, b"let foo(x: Int, y: Int) = ".to_vec(), ($body).as_bytes().to_vec(), vec![b';'], $msg);
    };
    (non_utf8, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_errors, $test_name, vec![], $body, vec![], $msg);
    };
    ($error_or_warning: ident, $test_name: ident, $prefix: expr, $body: expr, $suffix: expr, $msg: expr) => {
        #[test]
        fn $test_name() {
            let code = vec![
                $prefix,
                $body,
                $suffix,
            ].concat();
            let code_str = String::from_utf8_lossy(&code).to_string();
            let mut res = run(CompilerOption::test_runner(&code));

            let output = res.$error_or_warning();
            let msg_normalized = String::from_utf8_lossy(&normalize($msg)).to_string();
            let output_normalized = String::from_utf8_lossy(&normalize(&output)).to_string();

            // set this flag to see all the error messages and warnings
            let always_panic = false;

            if !output_normalized.contains(&msg_normalized) || always_panic {
                panic!(
                    "\n-----\nCode: {code_str}\n\nExpected: {}\n\nGot: \n{}\n-----\n",
                    $msg,
                    if output.is_empty() { String::from("<NO OUTPUT>") } else { output },
                );
            }
        }
    };
}

fn normalize(s: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len());

    for c in s.as_bytes() {
        if *c == b' ' {
            continue;
        }

        if b'A' <= *c && *c <= b'Z' {
            result.push(*c - b'A' + b'a');
        }

        else {
            result.push(*c);
        }
    }

    result
}

// error messages for invalid stmts
check_output!(stmt, err, import_test1, "import x, y,", "got nothing");
check_output!(stmt, err, import_test2, "import x, y from z, w;", "got `,`");
check_output!(stmt, err, import_test3, "import from x;", "got `from`");
check_output!(stmt, err, stmt_test1, "let foo<>() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test2, "let foo< >() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test3, "let foo<GenericName>() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test4, "let foo<GenericName, >() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test5, "def PI = 3;", "Do you mean `let`?");
check_output!(stmt, err, stmt_test6, "ket foo() = 3;", "you mean `let`?");
check_output!(stmt, err, stmt_test7, "let ZERO: int = 0;", "undefined name `int`");
check_output!(stmt, err, stmt_test8, "let lambda_test: Int = {
    # 한글 주석은 달아도 되지?
    let l = \\{x: Int, y: InvalidName, x + y + a + b};
    let a: Int = 3;

    l(a)
};", "undefined name `InvalidName`");
check_output!(stmt, err, stmt_test9, "let name_test: Int = {
    let 🦫 = \"beaver\";

    0
};", "got character '🦫'");
check_output!(stmt, err, stmt_test10, "let foo(
    #> Doc comment 1
    x: Int,

    #> Doc comment 2
    y: Int,

    #> Doc comment for nothing
) = x + y;", "stranded attribute");
check_output!(stmt, err, stmt_test11, "let foo() = {
    #> Doc comment 1
    let x = 3;

    #> Doc comment 2
    let y = 4;

    #> Doc comment for nothing
    x + y
};", "stranded attribute");
check_output!(stmt, err, nested_comment, "#! 123 #! 456 !#", "unterminated block comment");
check_output!(stmt, err, no_dependent_types1, "let foo(x: y, y: Int) = 0;", "dependent types");
check_output!(stmt, err, no_dependent_types2, "let foo(x: Int, y: x) = 0;", "dependent types");
check_output!(stmt, err, long_error_span, "
# Long Error Spans

\"
ErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorErrorError
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
                                                                                                                             Error
\"
", "expected a statement, got `\"...\"`");

check_output!(stmt, err, name_collision1, "let foo = 3; module foo;", "`foo` is bound multiple times");
check_output!(stmt, err, name_collision2, "import foo; module foo;", "`foo` is bound multiple times");
check_output!(stmt, err, name_collision3, "module foo; import foo;", "`foo` is bound multiple times");
check_output!(stmt, err, name_collision4, "import a.foo; import b.foo;", "`foo` is bound multiple times");
check_output!(stmt, err, name_collision5, "import x, x;", "`x` is bound multiple times");

// TODO
// check_output!(stmt, err, no_module2, "import invalid_module_name;", "module not found");
check_output!(stmt, err, name_collision6, "let foo = 3; let struct foo = { n: Int };", "`foo` is bound multiple times");
check_output!(stmt, err, kind_error_for_func_return_type, "let add(x: Int, y: Int) -> Int = x + y;", "try `:` instead");

// TODO: the compiler thinks `>` is an infix operator following `T`
// check_output!(stmt, err, kind_error_for_generic, "let unwrap<T>(option: Option<T>): T = match option { Option.Some(v) => v, Option.None => panic };", "_______");

// TODO: the compiler thinks `E` is a name of an argument
// check_output!(stmt, err, kind_error_for_generic2, "let unwrap<T>(result: Result<T, E>): T = match result { Result.Ok(v) => v, Result.Error(e) => panic };", "_______");

// non-utf8 inputs
check_output!(non_utf8, non_utf8_comment, make_non_utf8("# U\nlet main = 123;"), "invalid utf-8");
check_output!(non_utf8, non_utf8_ident, make_non_utf8("let main = {let U = 1; let x = 2; x + y};"), "invalid utf-8");
check_output!(non_utf8, non_utf8_string, make_non_utf8("let main = \"U\";"), "invalid utf-8");

// error messages for invalid exprs
check_output!(expr, err, expr_test1, "1...3.", "invalid literal: `...`");
check_output!(expr, err, expr_test2, "1...", "invalid literal: `...`");
check_output!(expr, err, expr_test3, "1 + ", "expected an expression");
check_output!(expr, err, expr_test4, "x.(x)", "name of a field must be an identifier.");
check_output!(expr, err, expr_test5, "[1, 2, x[]]", "got nothing");
check_output!(expr, err, expr_test6, "[(), {), ]", "unclosed delimiter");
check_output!(expr, err, expr_test7, "[(), {}, ]", "got nothing");
check_output!(expr, err, expr_test8, "[1, 2, 3, 4", "unclosed delimiter");
check_output!(expr, err, expr_test9, "if x { 0 } else { }", "got nothing");

// TODO
// check_output!(expr, err, expr_test10, "if x > y { x } * 2", "____");
// check_output!(expr, err, expr_test11, "if x > y { x }", "____");

check_output!(expr, err, expr_test12, "match {}", "got nothing");  // it expects `match { value } { arms }`
check_output!(expr, err, expr_test13, "match x {}", "got nothing");
check_output!(expr, err, expr_test14, "{let a = 3; let b = 4;}", "got nothing");
check_output!(expr, err, expr_test15, "{100 100}", "got `100`");
check_output!(expr, err, expr_test16, "[100 100]", "got `100`");
check_output!(expr, err, expr_test17, "[100 100, 100 100]", "got `100`");
check_output!(expr, err, expr_test18, "x[100 100]", "got `100`");
check_output!(expr, err, expr_test19, "(100 100)", "got `100`");
check_output!(expr, err, expr_test20, "foo(100 100)", "got `100`");
check_output!(expr, err, expr_test21, "한글넣으면죽음?", "got character '한'");
check_output!(expr, err, expr_test22, "{}", "got nothing");
check_output!(expr, err, expr_test22_2, "{{}}", "got nothing");
check_output!(expr, err, expr_test23, "f'\\{x} + \\{y} = \\{x + y}'", "single quotes");
check_output!(expr, err, expr_test24, "f\"ABC \\{}\"", "empty format-string");
check_output!(expr, err, expr_test25, "f\"ABC \\{1 + }\"", "got nothing");
check_output!(expr, err, expr_test26, "f\"ABC \\{ [][]}\"", "got nothing");
check_output!(expr, err, expr_test27, "(b \"ABC 한글 DEF\")", "got `\"...\"`");
check_output!(expr, err, expr_test28, "(f \"\\{a} + \\{b} = \\{a + b}\")", "add `f`");
check_output!(expr, err, expr_test29, "[0, 1, 2, 3] `10 1", "field modifier without");
check_output!(expr, err, expr_test30, "\\{x: Int, x: Int, x + x}", "`x` is bound multiple times");
check_output!(expr, err, expr_test31, "{let x = 3; let x = 4; x + x}", "name `x` is bound multiple times");
check_output!(expr, err, expr_test32, "   #!#  # Unfinished Comment", "unterminated block comment");
check_output!(expr, err, expr_test33, "f(x[..4])", "like `0..`");
check_output!(expr, err, expr_test34, "  {#!\n\n\n!#  }", "got nothing");
check_output!(expr, err, expr_test35, "match x {0..~ => 0, 1..2 => 3}", "inclusive range with an open end");
check_output!(expr, err, expr_test36, "Foo {}", "please provide fields");
check_output!(expr, err, expr_test37, "{let x = 3; let y = 4; x + y;}", "remove this `;`");
check_output!(expr, err, expr_test38, "", "expected an expression");
check_output!(expr, err, expr_test39, "'abc'", "too long character");
check_output!(expr, err, expr_test40, "match x { 0..'9' => 1, _ => 2, }", "got type `Char`");
check_output!(expr, err, expr_test41, "match x { 0..0.1 => 1, _ => 2, }", "got type `Ratio`");
check_output!(expr, err, expr_test42, "match x { 0..() => 1, _ => 2, }", "got type `Tuple`");
check_output!(expr, err, expr_test43, "match x { 0..0 => 0, _ => x }", "nothing can match this pattern");
check_output!(expr, err, expr_test44, "match x { 0.1..0.1 => 0, _ => x }", "nothing can match this pattern");
check_output!(expr, err, expr_test45, "match x { 2..1 => 0, _ => x }", "nothing can match this pattern");
check_output!(expr, err, expr_test46, "0bffff", "got character 'f'");
check_output!(expr, err, expr_test47, "{let generic<T, U> = T; generic}", "generic parameter not allowed");
check_output!(expr, err, expr_test48, "
    {
        let pattern ($x, ($y, $z)) = (0, (1, 2));
        let z = 10;

        x + y + z
    }", "`z` is bound multiple times");
check_output!(expr, err, expr_test49, "{let ($x, $y) = (0, 1); x}", "use `let pattern`");
check_output!(expr, err, expr_test50, "{let pattern ($x, .., ..) = (0, 1, 2, 3, 4); x}", "multiple shorthands");
check_output!(expr, err, expr_test51, "[[1, 2, 3, 4[], 5]]", "got nothing");

// TODO
// check_output!(expr, err, expr_test51, "{let pattern ($x .. $y) = (0, 1, 2); x}", "TODO: tell the user kindly that there should be a comma");

check_output!(expr, err, expr_test52, "{let x = 3\nlet y = 4\n x}", "use `;` before the keyword `let`");
check_output!(expr, err, expr_test53, "match x { 1.5..1.4 => 0, _ => x }", "unmatchable pattern");
check_output!(expr, err, expr_test54, "match x { 9.4..1.15 => 0, _ => x }", "unmatchable pattern");
check_output!(expr, err, expr_test55, "\"\\l\"", "try `\\\\l`");
check_output!(expr, err, expr_test56, "match \"abc\" { \"a\"..~\"c\" => 0, _ => 1 }", "inclusive range");
check_output!(expr, err, expr_test57, "match \"abc\" { \"a\"..(\"c\": String) => 0, _ => 1 }", "type annotation not allowed");
check_output!(expr, err, expr_test58, "match \"abc\" { \"a\"..($c @ \"c\") => 0, _ => 1 }", "name binding not allowed");

check_output!(expr, err, expr_test59, "match ((), ()) {($x @ (), $y @ ()) | (_, _) => (), _ => ()}", "name `x` not bound in all patterns");
check_output!(expr, err, expr_test60, "{let x = y; let y = x; 100}", "a cycle in local values");
check_output!(expr, err, expr_test61, "{let x = x + 1; 100}", "`x` is referencing itself");

// TODO: Type errors are not implemented yet
// check_output!(expr, err, expr_test59, "match \"abc\" { b\"a\"..\"c\" => 0, _ => 1 }", "------");
// check_output!(expr, err, expr_test60, "match \"abc\" { b\"a\"..3 => 0, _ => 1 }", "------");

check_output!(expr, err, fstring1, "f\"\\{1 + 3\"", "unterminated `\\{`");
check_output!(expr, err, fstring2, "\"\\{1 + 3}\"", "add `f`");
check_output!(expr, err, fstring3, "\'\\{1 + 3}\'", "use double quote");
check_output!(expr, err, fstring4, "b\"\\{1 + 3}\"", "format-string with a prefix `b`");

check_output!(expr, err, two_elses, "if 0 == 0 { 0 } else { 1 } else { 2 }", "got `else`");
check_output!(expr, err, branch_without_cond, "if 0 == 0 { 0 } else if { 1 } else { 2 }", "missing a condition");

check_output!(expr, err, errors_with_macros1, "@[]()", "expected an identifier, got nothing");
check_output!(expr, err, errors_with_macros2, "@[]", "expected `(");
check_output!(expr, err, errors_with_macros3, "@[abc]", "expected `(");

check_output!(expr, err, wrong_binding_order1, "match 1 { 1 @ $x => 1, _ => 0 }", "to bind a name to a pattern");

// TODO
// check_output!(expr, err, wrong_binding_order2, "match foo() { Foo { x: $x, y: $y } @ $x => 1, _ => 0 }", "to bind a name to a pattern");
check_output!(expr, err, wrong_binding_order3, "match foo() { Foo { x: 1 @ $x, y: 2 } => 1, _ => 0 }", "to bind a name to a pattern");
check_output!(expr, err, wrong_binding_order4, "{ let pattern (1, 2, 3) @ $x = (1, 2, 3); x }", "to bind a name to a pattern");
check_output!(expr, err, wrong_binding_order5, "{ let pattern (1, 2, 3 @ $x) = (1, 2, 3); x }", "to bind a name to a pattern");
check_output!(expr, err, wrong_binding_order6, "{ let pattern (1, 2 @ $x, 3) = (1, 2, 3); x }", "to bind a name to a pattern");

check_output!(expr, err, refutable_pattern1, "{ let pattern ($x @ 1, $y @ 2) = (10, 20); x + y }", "refutable pattern");

// TODO
// check_output!(expr, err, logical_and_to_ifs, "if True || 3 { 0 } else { 1 }", "TODO: emit a nice type error");

// warnings for stmts
check_output!(stmt, warn, stmt_warn_test1, "let foo(x: Int, y: Int, z: Int): Int = x + y;", "unused function argument: `z`");
check_output!(stmt, warn, stmt_warn_test2, "let foo<T>(x: Int, y: Int): Int = x + y;", "unused generic: `T`");
check_output!(stmt, warn, stmt_warn_test3, "let Int: Type = 0;", "prelude `Int`");
check_output!(stmt, warn, stmt_warn_test4, "import x, y, z;", "unused import: `x`");
check_output!(stmt, warn, stmt_warn_test5, "import x, y, z;", "unused import: `y`");
check_output!(stmt, warn, stmt_warn_test6, "import x, y, z;", "unused import: `z`");

// warnings for exprs
check_output!(expr, warn, expr_warn_test1, "{let x = 3; 0}", "unused local name binding");
check_output!(expr, warn, expr_warn_test2, "match x { $x @ $y @ 0 => 1, _ => 2, }", "multiple name bindings");
check_output!(expr, warn, expr_warn_test3, "f\"1234\"", "nothing to evaluate");
check_output!(expr, warn, expr_warn_test4, "{{5}}", "unnecessary parenthesis");
check_output!(expr, warn, expr_warn_test5, "match x { 0..~0 => 0, _ => x }", "`0..~0` is just `0`");
check_output!(expr, warn, expr_warn_test6, "match x { 0.1..~0.1 => 0, _ => x }", "`0.1..~0.1` is just `0.1`");
check_output!(expr, warn, expr_warn_test7, "match x { 1..2 => 1, _ => x }", "`1..~1` is just `1`");
check_output!(expr, warn, expr_warn_test8, "{let pattern ($xx, $yy) = (0, 1); xx}", "unused local name binding");
check_output!(expr, warn, expr_warn_test9, "
    {
        let pattern ($x, ($y, $z)) = (0, (1, 2));
        let w = 10;

        x + y + z
    }", "unused local name binding in a scoped let: `w`");
check_output!(expr, warn, expr_warn_test10, "{let pattern ($x @ _, $y) = (0, 1); y}", "name binding on wildcard");
check_output!(expr, warn, expr_warn_test11, "match (1, 2) {($aa @ (1 | 2), $bb @ (3 | 4)) => aa, _ => 0}", "unused name binding in match arm: `bb`");
check_output!(expr, warn, expr_warn_test12, "match (1, 2) { ($x @ ($y @ 1 | $z @ 2), $a @ ($b @ 1 | $c @ 2)) => 0, _ => 1 }", "multiple name bindings on a pattern");
check_output!(expr, warn, expr_warn_test13, "f\"\\{{let x = 3; 4}}\"", "unused local name binding");
check_output!(expr, warn, expr_warn_test14, "{let x = 3; let y = x; 100}", "`x` is used by another value");

fn make_non_utf8(s: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len() + 4);

    for c in s.as_bytes().iter() {
        if *c == b'U' {
            result.push(200);
            result.push(200);
            result.push(200);
        }

        else {
            result.push(*c);
        }
    }

    result
}
