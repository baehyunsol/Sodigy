Impure functions

TODO: In which contexts can we use impure functions?

# run

`run(x: T, y: U): T`

It gets 2 inputs `x` and `y`, and returns `x`. `x` and `y` are guaranteed to be strictly evaluated. `x` is evaluated before `y`.

TODO: combine multiple `run`s and make a macro

TODO: make one that returns `y` instead of `x`

# print

`print(x: T): T`

It prints `x` and returns `x`.

# sleep

`sleep(millisecond: Int): Int`

It sleeps and returns `0`.

# random

two types of random functions: both are impure, but one has an internal state, and the other doesn't

the one with an internal state has helper function: set the state

# time

one that returns elapsed microseconds since UNIX epoch

# panic

`panic` is not an impure function! Its return type is `Never`, which is a subtype of every other type. That means you can call this function in almost everywhere.

Thanks to this function, `assert` and `assert_eq` are also functions.

---

# Examples

## `test.log`

```
@[run_debug](
    @[run_last_debug](
        print(f"f(x: \{x}, y: \{y})"),
        indent_in(),
        store_tmp_value(f(x, y)),
        peek_tmp_value(),
    ),
    indent_out(),
    print(f"returned with result \{res}"),
    pop_tmp_value(),
)
```

TODO: we cannot do this if `f` tail-calls itself.

## `test.after` and `test.before`

```
let foo(x, y) = @[run_debug](
    @[run_last_debug](
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
@[run_last_debug](
    if foo != x {
        @[run](
            print(f"assertion failure: \{foo} != \{x}"),
            set_test_failure_flag(true),
        )
    } else {
        0
    },
    if bar != y {
        @[run](
            print(f"assertion failure: \{bar} != \{y}"),
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
let foo(x, y) = @[run_last_debug](
    print(f"foo(x: \{x}, y: \{y})"),
    input(),  # waits for the user to continue
    foo(x, y),
);
```
