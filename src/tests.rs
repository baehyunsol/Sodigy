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
            let res = compile_input(
                format!(
                    "{}{}{}",
                    $prefix, $body, $suffix,
                ).as_bytes().to_vec()
            );

            if !res.$error_or_warning().contains($msg) {
                panic!(
                    "\n-----\nExpected: {}\n\nGot: \n{}\n-----\n",
                    $msg,
                    res.$error_or_warning(),
                );
            }
        }
    };
}

// error messages for invalid stmts
// TODO: first implement the parser
// "use A.{B, C} as D;",
// "use A.{};",
// "use A.{,};",
// "use A.{B, C;};",
// "use A as B as C;",
// check_output!(stmt, err, stmt_test1, "use A.{B, C} as D;", "TODO");

check_output!(stmt, err, stmt_test1, "def foo<>() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test2, "def foo< >() = 3;", "remove angle brackets");
check_output!(stmt, err, stmt_test3, "def foo<GenericName>() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test4, "def foo<GenericName, >() = generic_name;", "similar name exists");
check_output!(stmt, err, stmt_test5, "let PI = 3;", "Try `def`");
check_output!(stmt, err, stmt_test6, "fef foo() = 3;", "you mean `def`?");
check_output!(stmt, err, refuse_dependent_types, "def foo(x: y, y: Int) = 0;", "dependent types");

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
check_output!(expr, err, expr_test10, "if x > y { x } * 2", "TODO");
check_output!(expr, err, expr_test11, "if x > y { x }", "TODO");
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
check_output!(expr, err, expr_test23, "f'{x} + {y} = {x + y}'", "single quotes");
check_output!(expr, err, expr_test24, "f\"ABC {}\"", "empty format-string");
check_output!(expr, err, expr_test25, "f\"ABC {1 + }\"", "got nothing");
check_output!(expr, err, expr_test26, "f\"ABC { [][]}\"", "got nothing");
check_output!(expr, err, expr_test27, "f\"{1\"", "TODO");
check_output!(expr, err, expr_test28, "(b \"ABC 한글 DEF\")", "got `\"...\"`");
check_output!(expr, err, expr_test29, "(f \"{a} + {b} = {a + b}\")", "got `\"...\"`");
check_output!(expr, err, expr_test30, "[0, 1, 2, 3] `10 1", "field modifier without");
check_output!(expr, err, expr_test31, "\\{x: Int, x: Int, x + x}", "TODO");
check_output!(expr, err, expr_test32, "{let x = 3; let x = 4; x + x}", "TODO");
check_output!(expr, err, expr_test33, "   ##!##  # Unfinished Comment", "unterminated block comment");
check_output!(expr, err, expr_test34, "f(x[..4])", /*L*/ "ike `0..`");
check_output!(expr, err, expr_test35, "  {##!\n\n\n!##  }", "got nothing");

// warnings for stmts
check_output!(stmt, warn, stmt_warn_test1, "def foo(x: Int, y: Int, z: Int): Int = x + y;", "unused function argument: `z`");
check_output!(stmt, warn, stmt_warn_test2, "def foo<T>(x: Int, y: Int): Int = x + y;", "unused generic: `T`");

// warnings for exprs
check_output!(expr, warn, expr_warn_test1, "{let x = 3; 0}", "TODO");
