# Sodigy Testbench

You can find script files in `tests/runner/`. You can run the tests with the scripts. The scripts are currently written in Python, but I'll rewrite them in Sodigy someday.

Go to `tests/runner/` and run `python3 main.py all`. It'll run the tests and write the result json file to `tests/results/`.

The test cases are in `tests/single-file/` and `tests/multi-files/` (WIP).

## Single-file

In this directory, you'll see bunch of `.sdg` files. Each file is a test case.

The test runner will run `sodigy new sodigy-test`, copy a test file to `sodigy-test/src/lib.sdg`, and run `sodigy test`. The test runner will make sure that 1) the code compiles and 2) all the assertions succeed.

Sometimes you want to assert that a file does not compile. Or sometimes you want to assert that a file emits specific kind of errors. The test runner recognizes special kind of macro. Lines that start with `//#` (so it's a line comment in Sodigy) are macros.

In each macro, you assert something using python syntax.

```sodigy
//# assert len(errors) == 1, "there should be exactly 1 error"
//# assert len(warnings) == 0, "there should be no warnings"
//# assert errors[0].index == 350, f"expected error-350, got error-{errors[0].index}"

type T = [T];
```

Then, the test runner will define a python function `def expect(status: str, errors: [Error], warnings: [Error], success: bool, test_error: bool, compile_error: bool, misc_error: bool, timeout: bool)` and copy-paste the assertions to the body of the function. The test runner will call `expect` with the result of `sodigy test`. If `expect` throws an error, the test fails.

`errors: [Error]` is a list of compile-errors, not runtime assertions (of Sodigy). You can find its definition at `tests/runner/error.py`.

## Multi-files

TODO
