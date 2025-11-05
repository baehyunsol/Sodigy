# Some Design Decisions

Decisions with smaller numbers are more important.

## 1. Everything has to be purely functional.

There are 2 languages: the core language, which is pure, and a script language, which is impure.

You write the pure logic (which must be 99% of your project) in the core language, and to the impure IO (the remaining 1%) with the script language.

## 2. The language has to be Rust-like.

It has 2 benefits.

1. Rust users can easily learn Sodigy.
2. Rust is battle-tested. It's not perfect, but is better than designing a language from scratch.

I'm trying to make Sodigy as Rust-like as possible as long as I can keep it pure.

## 3. Compile-time Checks

The compiler has to catch as many errors as possible at compile time.

Generating nice error messages is more important than quick compile time.

## 4. Runtime performance

Sodigy is not a system programming language. It has to be a simple and abstracted language as long as it is faster than Python.

## 5. Sodigy uses arbitrary width integers.

I like arbitrary width integers. That's it. If you want 32-bit integers or 64-bit integers, this is not your language.

## 6. It's a language for medium~large projects.

You need at least 3 files to run a Sodigy program (`sodigy.toml`, `src/lib.sdg`, `src/main.sdgsh`), so it's not a good idea to use Sodigy for a very simple program, which can be done in a few lines.

In order to run a Sodigy program, you have to create a project (`sodigy new`), write code, and run. It spawns child processes and creates a lot of intermediate files. So it's not a good idea to embed Sodigy in another language.

It'd be lovely if we can build GUI programs or games with Sodigy, but we're not there yet.
