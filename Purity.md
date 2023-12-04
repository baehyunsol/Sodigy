Impure functions.

# run

`run(x1: T, x2: U, .., xn: V): T`

`run` is a function that helps you run code synchronoushly.

- Impure
- Input: arbitrary number of inputs (at least one)
- Output: returns the first argument
- Behavior
  - It's guaranteed that all the arguments are evaluated (executed).
  - It's guaranteed that an evaluation (execution) of an argument starts after the evaluation (execution) of the previous argument is finished.

# run_last

`run_last(x1: T, x2: U, .., xn: V): V`

`run_last` is like `run` but returns the last input, not the first one. You can make arbitrary sequence of execution by combining `run`s and `run_last`s.

- Impure
- Input: arbitrary number of inputs (at least one)
- Output: returns the last argument
- Behavior
  - It's guaranteed that all the arguments are evaluated (executed).
  - It's guaranteed that an evaluation (execution) of an argument starts after the evaluation (execution) of the previous argument is finished.

# run_debug

`run_debug(x1: T, x2: U, .., xn: V): T`

`run_debug` is like `run` but only works in debug mode. It's optimized away in release mode and becomes `x1`.

- Pure (only works in debug mode)
  - It's optimized away in release mode.
- Input: arbitrary number of inputs (at least one)
- Output: returns the first argument
- Behavior
  - It's guaranteed that all the arguments are evaluated (executed).
  - It's guaranteed that an evaluation (execution) of an argument starts after the evaluation (execution) of the previous argument is finished.

# run_last_debug

`run_last_debug(x1: T, x2: U, .., xn: V): V`

`run_last_debug` is like `run_last` but only works in debug mode. It's optimized away in release mode and becomes `xn`.

- Pure (only works in debug mode)
  - It's optimized away in release mode.
- Input: arbitrary number of inputs (at least one)
- Output: returns the last argument
- Behavior
  - It's guaranteed that all the arguments are evaluated (executed).
  - It's guaranteed that an evaluation (execution) of an argument starts after the evaluation (execution) of the previous argument is finished.

# inspect

`inspect(x: T): T`

`inspect` prints a value. It only works in debug mode. It's optimized away in release mode and becomes `x`.

- Pure (only works in debug mode)
  - It's optimized away in release mode.
- Input: a single value
- Output: returns the input
- Behavior
  - It prints `x` and returns `x`.

```
run_last_debug(
    print(f"{x}"),
    x,
)
```

# print

`print(x: T): T`

`print` is like `inspect`, but is never optimized away.

- Impure
- Input: a single value
- Output: returns the input
- Behavior
  - It prints `x` and returns `x`.

# sleep

`sleep(millisecond: Int): Int`

- Impure
- Input: an integer (millisecond to sleep)
- Output: 0 when successful, non-zero otherwise
- Behavior
  - It sleeps.

# random

two types of random functions: both are impure, but one has an internal state, and the other doesn't

the one with an internal state has helper function: set the state

# time

one that returns elapsed microseconds since UNIX epoch

# panic

Unlike others in this document, `panic` is a function! Its return type is `Never`, which is a subtype of every other type. That means you can call this function in almost everywhere.

Thanks to this function, `assert` and `assert_eq` are also functions.

---

# Examples

## `test.log`

```
run_debug(
    run_last_debug(
        print(f"f(x: {x}, y: {y})"),
        indent_in(),
        store_tmp_value(f(x, y)),
        peek_tmp_value(),
    ),
    indent_out(),
    print(f"returned with result {res}"),
    pop_tmp_value(),
)
```

TODO: we cannot do this if `f` tail-calls itself.

## `test.after` and `test.before`

```
let foo(x, y) = run_debug(
    run_last_debug(
        \{assert(x > 0)}(),
        store_tmp_value(foo_real(x, y)),
        peek_tmp_value(),
    ),
    \{res, check(res)}(pop_tmp_value()),
);
```

## `test.eq`

```
@test.eq(x)
let foo = baz();

@test.eq(y)
let bar = baz();
```

```
# It modifies the main function
run_last_debug(
    if foo != x {
        run(
            print(f"assertion failure: {foo} != {x}"),
            set_test_failure_flag(true),
        )
    } else {
        0
    },
    if bar != y {
        run(
            print(f"assertion failure: {bar} != {y}"),
            set_test_failure_flag(true),
        )
    } else {
        0
    },
    if get_test_failure_flag() { panic() } else { 0 },
    main(),
)
```

## `test.breakpoint`

```
let foo(x, y) = run_last_debug(
    print(f"foo(x: {x}, y: {y})"),
    input(),  # waits for the user to continue
    foo(x, y),
);
```
