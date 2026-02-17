# Some Design Decisions

Lowered numbered decisions have higher priority.

## 1. Everything has to be purely functional.

Sodigy strictly distinguishes between pure functions and impure functions via `pure` keyword.

Sodigy has no notion of "execution" or "evaluation". There are only values. An expression like `square(5)` doesn't "evaluate to" `25`. It simply "is" `25`.

Panicking is not considered a "behavior" in Sodigy. It only cares about what value a function returns, or whether it terminates. The optimizer may transform a panicking program into a non-panicking one.

## 2. Compile-time Checks

The compiler has to catch as many errors as possible at compile time. Also, it has to give as nice error messages (and warnings) as possible to the programmer.

That's why Sodigy does not have conditional compilations (like `cfg` in Rust) because conditional compilations might hide some errors.

## 3. The language has to be Rust-like.

It has 2 benefits.

1. Rust users can easily learn Sodigy.
2. Rust is battle-tested. It's not perfect, but getting inspirations from Rust is better than designing a language from scratch.

When I design a new feature, the first question I ask is "How does Rust solve this problem?".

## 4. Runtime performance

Sodigy is not a system programming language. Making Sodigy as bare metal as C is a non-goal. I want Sodigy to be 2~3 times faster than Python.

## 5. Number system

Sodigy uses arbitrary width integers. I want to avoid all the uglinesses from fixed size integers. You know, Python uses arbitrary width integers but people still use it!
