# Sodigy Testbench

You can find script files in `tests/runner/`. You can run the tests with the scripts. The scripts are currently written in Python, but I'll rewrite them in Sodigy someday.

The test cases are in `tests/single-file/` and `tests/multi-files/` (WIP).

## Single-file

In this directory, you'll see bunch of `.sdg` files. Each file is a test case.

The test runner will run `sodigy new sodigy-test`, copy a test file to `sodigy-test/src/lib.sdg`, and run `sodigy test`. The test runner will make sure that 1) the code compiles and 2) all the assertions succeed.

Sometimes you want to assert that a file does not compile. Or sometimes you want to assert that a file emits specific kind of errors. The test runner recognizes special kind of macro. The macro starts with `/*<expect>`, ends with `</expect>*/` (so it's a block comment in Sodigy), and must be placed at the top of the file.

In the macro, you define a python function `def expect(result):`. If a macro is found, the test runner will pass the run_result to the function you defined, and will see if the function throws an error or not. Below is an example.

```sodigy
/*<expect>

def expect(result):
    assert len(result.errors) == 1, "there should be exactly 1 error"
    assert len(result.warnings) == 0, "there should be no warnings"
    assert result.errors[0].index == 350, f"expected error-350, got error-{result.errors[0].index}"

</expect>*/

type T = [T];
```

In the example, the first 8 lines are macro: it defines `expect` function in Python. The last line is the actual test, in Sodigy.

After the test runner compiles the sodigy code, the runner will pass the result to `expect` in the macro. `expect` will make sure that the compiler emits only 1 error, and the error is error-350.

## Multi-files

TODO
