# Sodigy Test Harness

You can find the test runner in `tests/runner/`. The runner is written in Rust, and you need cargo to build the test runner.

In order to run the full harness, you also need git installed because it runs `std::process::Command::new("git")`.

```sh
# Runs "compile-and-run" test suite (full suite).
cargo run -- cnr;

# Runs "compile-and-run" test suite, but only cases that have "foo" in their names.
cargo run -- cnr foo;

# Runs "crates" test suite.
cargo run -- crates;

# Runs all the test suites.
cargo run -- all;
```

## Test Suites

### compile-and-run

As of now, this is the main test suite. It is very similar to rust's [compiletest](https://rustc-dev-guide.rust-lang.org/tests/compiletest.html).

#### Add cases (single-file)

If your test case consists of a single file, add the file to `tests/compile-and-run/`. The file name must start with the name of the test-case, and its extension must be `.sdg`. There should be no dot (`.`) character in the test-case name.

For example, if you want to add a test named "foo-1", create a file `tests/compile-and-run/foo-1.sdg`. The test runner will iterate the files in `tests/compile-and-run/` and collect files whose extension is `.sdg`.

You can add expected-output files. For example, if you want to check the compiler-stderr of "foo-1", you can add `tests/compile-and-run/foo-1.compile.stderr`. The file must start with the test-case name. You can read more about expected-output files later.

#### Add cases (multi-file)

Let's say you want to add a test case "foo-2", and you want it to have multiple files (multiple modules). You have to run `sodigy new foo-2` inside `tests/compile-and-run/`. The command will create a directory `tests/compile-and-run/foo-2/`. The test-runner treats each directory inside `tests/compile-and-run` as a test case with the same name.

You can also add expected-output files. Create `tests/compile-and-run/foo-2.compile.stderr` to check the compiler output.

#### Expected Output

There are 4 possible extensions: `.compile.stdout`, `.compile.stderr`, `.run.stdout` and `.run.stderr`. For example, `tests/compile-and-run/foo.compile.stderr` is an expected-output of the stderr of the compilation of `tests/compile-and-run/foo.sdg` or `tests/compile-and-run/foo/`. Each test case consists of 2 stages: it first compiles the sodigy code, then it checks the assertions in the sodigy code. The output of the first stage is matched against `.compile.xxxxxx` and the second stage is matched against `.run.xxxxxx`.

It normalizes the output before comparison. ANSI terminal colors are removed, and it trims each line.

The most naive way to create an expected-output file is to copy-paste output of the compiler. Then the test runner will check if the compiler emits exactly the same output. But in most cases, there are details that you want to ignore. For example, error messages might change slightly when the compiler is updated. You only want the error index, which doesn't change.

An expected-output file accepts special syntaxes. If a line is 6 dots (`......`), it matches arbitrary number of lines (can be 0). So, the below expected-output file checks 1) if the first line of the output is "Hello, World" and 2) the last line of the output is "Goodbye, World". It ignores the lines in between.

```
Hello, World
......
Goodbye, World
```

3 dots (`...`) matches arbitrary number of characters. For example, `error (e-0350)...` matches a line that starts with "error (e-0350)". Any characters can follow. You can use this syntax multiple times in a line. For example, `...foo...bar...` matches a line that contains "foo" and "bar", and "bar" must follow "foo".

#### Directives

You can add directives to the test file. If the case is multi-file, you have to add directives to `src/lib.sdg` of the project. A directive is a line that starts with `//%`, and followed by commands.

- `//% compile-pass`
  - This test case must be successfully compiled.
- `//% compile-fail`
  - This test case must not be successfully compiled.
- `//% run-pass`
  - This test case must be successfully compiled, and assertions in the test case must all succeed.
- `//% run-fail`
  - This test case must be successfully compiled, and there must be a failing assertion in the test case.
- `//% compile-error > 3`
  - There must be more than 3 compile errors.
  - You can use 6 operators: `>`, `>=`, `<`, `<=`, `==`, `!=`
- `//% compile-warning > 3`
  - There must be more than 3 compiler warnings.
  - You can use 6 operators: `>`, `>=`, `<`, `<=`, `==`, `!=`
- `//% run-error > 3`
  - There must be more than 3 failing assertions.
  - You can use 6 operators: `>`, `>=`, `<`, `<=`, `==`, `!=`

TODO: If an assertion's name starts with "must-fail", it must fail.

### crates

It runs `cargo test`, `cargo test --release` and `cargo doc` in every crates in `crates/`.

### Full test suite

By running `cargo run -- all`, it runs all the test suites. It'll dump the result in a json format. It'll create a json file in the current working directory (where you run `cargo`). It'll create a copy in the `tests/log/`.

The result file name looks like `sodigy-test-1f97fd703-linux.json`. It uses git commit hash and os name (linux | mac | windows) to identify itself. So, running the test suite in a dirty repositroy doesn't make much sense. Please make sure to commit changes before running the test.
