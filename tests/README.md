# Sodigy Test Harness

TODO: rewrite the test runner in Sodigy

You can find the test runner in `tests/runner/`. The runner is written in Rust, and you need cargo to build the test runner.

In order to run the full harness, you also need git installed because it runs `std::process::Command::new("git")`.

```sh
# Runs "compile-and-run" test suite (full suite).
cargo run -- cnr;

# Runs "compile-and-run" test suite, but only cases that have "foo" in their names.
cargo run -- cnr foo;

# Runs "crates" test suite.
cargo run -- crates;

# Runs all test suites.
# It'll create a json file with the tests' result.
# Make sure that your repository is clean before running the tests
# because the json file uses commit hash to identify itself.
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

TODO: DOC

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
