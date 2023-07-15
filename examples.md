```
# TODO: type signatures of functors
def adder(n: Int): Fn(Int, Int) = \{x: Int, x + n};

@test.eq([8, 16])
def adder_test: List(Int) = [adder(3)(5), adder(7)(9)];
```

```
@memoize
def ackermann(m: Int, n: Int): Int = if m == 0 {
    n + 1
} else if n == 0 {
    ackermann(m, 1)
} else {
    ackermann(m, ackermann(m + 1, n))
};

@memoize
def fibonacci(n: Int): Int = if n < 2 {
    1
} else {
    fibonacci(n - 1) + fibonacci(n - 2)
};

@test.eq(125)
def ackermann_test: Int = ackermann(3, 4);

@test.eq(10946)
def fibonacci_test: Int = fibonacci(20);
```

```
@test.eq(10946)
def fibo_lambda: Int = {
    fibo = \{n, if n < 2 { 1 } else { fibo(n - 1) + fibo(n - 2) }};

    fibo(20)
};
```