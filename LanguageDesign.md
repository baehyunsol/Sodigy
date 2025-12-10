# Some Design Decisions

Decisions with smaller numbers are more important.

## 1. Everything has to be purely functional.

There are 2 languages: the core language, which is pure, and a script language, which is impure.

You write the pure logic (which must be 99% of your project) in the core language, and to the impure IO (the remaining 1%) with the script language.

## 2. Compile-time Checks

The compiler has to catch as many errors as possible at compile time. Also, it has to give as nice error messages (and warnings) as possible to the programmer.

That's why Sodigy does not have conditional compilations (like `cfg` in Rust) because conditional compilations might hide some errors.

## 3. The language has to be Rust-like.

It has 2 benefits.

1. Rust users can easily learn Sodigy.
2. Rust is battle-tested. It's not perfect, but getting inspirations from Rust is better than designing a language from scratch.

When I design a new feature, the first question I ask is "How does Rust solve this problem?".

## 4. Runtime performance

Sodigy is not a system programming language. Making Sodigy as bare metal as C is a non-goal.

## 5. Number system

Sodigy uses arbitrary-width integers instead of fixed width integers and rational numbers instead of floating point numbers.

I chose arbitrary-width integers because... (TODO)

I chose rational numbers because...

1. We cannot implement floating point numbers in Sodigy. We have to add a lot of built-ins to the language if we want floating points. That'll make Sodigy less portable.
2. `0.1 + 0.2 == 0.3`
3. If fp performance matters (e.g. you want to multiply numbers 1 billion times), Sodigy is not for you. See #4. Sodigy is not CUDA/C/Rust.
