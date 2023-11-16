use super::compile_input;

macro_rules! check_output {
    (stmt, err, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_errors, $test_name, "", $body, "", $msg);
    };
    (expr, err, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_errors, $test_name, "def foo(x: Int, y: Int) = ", $body, ";", $msg);
    };
    (stmt, warn, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_warnings, $test_name, "", $body, "", $msg);
    };
    (expr, warn, $test_name: ident, $body: expr, $msg: expr) => {
        check_output!(concat_warnings, $test_name, "def foo(x: Int, y: Int) = ", $body, ";", $msg);
    };
    ($error_or_warning: ident, $test_name: ident, $prefix: expr, $body: expr, $suffix: expr, $msg: expr) => {
        #[test]
        fn $test_name() {
            let code = format!(
                "{}{}{}",
                $prefix, $body, $suffix,
            );
            let mut res = compile_input(
                code.as_bytes().to_vec()
            );
            let output = res.$error_or_warning();
            let msg_normalized = String::from_utf8_lossy(&normalize($msg)).to_string();
            let output_normalized = String::from_utf8_lossy(&normalize(&output)).to_string();

            // set this flag to see all the error messages and warnings
            let always_panic = false;

            if !output_normalized.contains(&msg_normalized) || always_panic {
                panic!(
                    "\n-----\nCode: {code}\n\nExpected: {}\n\nGot: \n{output}\n-----\n",
                    $msg,
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
check_output!(stmt, err, stmt_test1, "def foo<>() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test2, "def foo< >() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test3, "def foo<GenericName>() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test4, "def foo<GenericName, >() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test5, "let PI = 3;", "Try `def`");
check_output!(stmt, err, stmt_test6, "fef foo() = 3;", "you mean `def`?");
check_output!(stmt, err, no_dependent_types, "def foo(x: y, y: Int) = 0;", "dependent types");

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
check_output!(expr, err, expr_test10, "if x > y { x } * 2", "TODO ____");
check_output!(expr, err, expr_test11, "if x > y { x }", "TODO ____");
check_output!(expr, err, expr_test12, "match {}", "got nothing");  // it expects `match { value } { arms }`
check_output!(expr, err, expr_test13, "match x {}", "got nothing");
check_output!(expr, err, expr_test14, "{let a = 3; let b = 4;}", "got nothing");
check_output!(expr, err, expr_test15, "{100 100}", "got `100`");
check_output!(expr, err, expr_test16, "[100 100]", "got `100`");
check_output!(expr, err, expr_test17, "[100 100, 100 100]", "got `100`");
check_output!(expr, err, expr_test18, "x[100 100]", "got `100`");
check_output!(expr, err, expr_test19, "(100 100)", "got `100`");
check_output!(expr, err, expr_test20, "foo(100 100)", "got `100`");
check_output!(expr, err, expr_test21, "한글넣으면죽음?", "got `한`");
check_output!(expr, err, expr_test22, "{}", "got nothing");
check_output!(expr, err, expr_test22_2, "{{}}", "got nothing");
check_output!(expr, err, expr_test23, "f'{x} + {y} = {x + y}'", "single quotes");
check_output!(expr, err, expr_test24, "f\"ABC {}\"", "empty format-string");
check_output!(expr, err, expr_test25, "f\"ABC {1 + }\"", "got nothing");
check_output!(expr, err, expr_test26, "f\"ABC { [][]}\"", "got nothing");
check_output!(expr, err, expr_test27, "(b \"ABC 한글 DEF\")", "got `\"...\"`");
check_output!(expr, err, expr_test28, "(f \"{a} + {b} = {a + b}\")", "got `\"...\"`");
check_output!(expr, err, expr_test29, "[0, 1, 2, 3] `10 1", "field modifier without");
check_output!(expr, err, expr_test30, "\\{x: Int, x: Int, x + x}", "`x` is bound multiple times");
check_output!(expr, err, expr_test31, "{let x = 3; let x = 4; x + x}", "name `x` is bound multiple times");
check_output!(expr, err, expr_test32, "   ##!##  # Unfinished Comment", "unterminated block comment");
check_output!(expr, err, expr_test33, "f(x[..4])", "like `0..`");
check_output!(expr, err, expr_test34, "  {##!\n\n\n!##  }", "got nothing");
check_output!(expr, err, expr_test35, "match x {0..~ => 0, 1..2 => 3}", "inclusive range with an open end");
check_output!(expr, err, expr_test36, "Foo {}", "please provide fields");
check_output!(expr, err, expr_test37, "{let ($y, $z) = (0, 1); y}", "TODO ____");
check_output!(expr, err, expr_test38, "", "expected an expression");
check_output!(expr, err, expr_test39, "'abc'", "too long character");
check_output!(expr, err, expr_test40, "match x { 0..'9' => 1, _ => 2, }", "type error");
check_output!(expr, err, expr_test41, "match x { 0..0.1 => 1, _ => 2, }", "type error");
check_output!(expr, err, expr_test42, "match x { 0..() => 1, _ => 2, }", "type error");
check_output!(expr, err, expr_test43, "match x { 0..0 => 0, _ => x }", "nothing can match this pattern");
check_output!(expr, err, expr_test44, "match x { 0.1..0.1 => 0, _ => x }", "nothing can match this pattern");
check_output!(expr, err, expr_test45, "match x { 2..1 => 0, _ => x }", "nothing can match this pattern");

// warnings for stmts
check_output!(stmt, warn, stmt_warn_test1, "def foo(x: Int, y: Int, z: Int): Int = x + y;", "unused function argument: `z`");
check_output!(stmt, warn, stmt_warn_test2, "def foo<T>(x: Int, y: Int): Int = x + y;", "unused generic: `T`");
check_output!(stmt, warn, stmt_warn_test3, "def Int: Type = 0;", "prelude `Int`");

// warnings for exprs
check_output!(expr, warn, expr_warn_test1, "{let x = 3; 0}", "unused local name binding");
check_output!(expr, warn, expr_warn_test2, "match x { $x @ $y @ 0 => 1, _ => 2, }", "multiple name bindings");
check_output!(expr, warn, expr_warn_test3, "f\"{1\"", "unmatched");
check_output!(expr, warn, expr_warn_test4, "f\"1234\"", "nothing to evaluate");
check_output!(expr, warn, expr_warn_test5, "{{5}}", "unnecessary parenthesis");
check_output!(expr, warn, expr_warn_test6, "match x { 0..~0 => 0, _ => x }", "`0..~0` is just `0`");
check_output!(expr, warn, expr_warn_test7, "match x { 0.1..~0.1 => 0, _ => x }", "`1e-1..~1e-1` is just `1e-1`");
check_output!(expr, warn, expr_warn_test8, "match x { 1..2 => 1, _ => x }", "`1..~1` is just `1`");
