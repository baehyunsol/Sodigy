NOTE: nothing is implemented yet

SodigyC Test Runner automatically runs tests in this directory. Below is how it works.

The test runner searches for `test.json` inside this directory (and recursively). If it finds one, it runs a test according to the file.

## test.json

A test file consists of one or more tests. A test case is a JSON-object. If a file is a JSON-object, the file has one test case. If a file is an array of JSON-objects, the file has multiple test cases.

## JSON-object

A JSON-object represents a test case. There are 3 kinds of test cases: `run`, `compile` and `test`.

`run` runs a sodigy file in the dir, with an optional `stdin`. It checks whether the `stdout` and `stderr` from the run matches the conditions in the JSON.

`compile` compiles a sodigy file. You cannot give a `stdin` option. It checks whether the `stdout` and `stderr` from the compilation matches the conditions.

`test` runs `sodigy --test`.
