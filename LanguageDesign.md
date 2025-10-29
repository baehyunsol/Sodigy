# Some Design Decisions

Decisions with smaller numbers are more important.

## 1. Everything has to be purely functional.

There are 2 languages: the core language, which is pure, and a script language, which is impure.

You write the pure logic (which must be 99% of your project) in the core language, and to the impure IO (the remaining 1%) with the script language.

## 2. Runtime performance

Sodigy is not a system programming language. It has to be a simple and abstracted language as long as it is faster than Python.

## 3. The language has to be Rust-like.

It has 2 benefits.

1. Rust users can easily learn Sodigy.
2. Rust is battle-tested. It's not perfect, but is better than designing a language from scratch.

I'm trying to make Sodigy as Rust-like as possible unless it violates principle 1 or 2.

# 4. It's a language for building CLI tools.

You need at least 3 files to run a sodigy program (`sodigy.toml`, `src/lib.sdg`, `src/main.sdgsh`), so it's not a good idea to use Sodigy for a very simple program, which can be done in a few lines.

In order to run a Sodigy program, you have to create a project (`sodigy new`), write code, and run. It spawns child processes and creates a lot of intermediate files. So it's not a good idea to embed sodigy in another language.
