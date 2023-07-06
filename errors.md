Compiler should not panic in any case. If it panics, that's an error.

- Rules for `unwrap`, `expect`, and `panic!`
  - `unwrap` should be avoided in any case.
  - If you want to unwrap something, use `expect("Internal Compiler Error XXXXXXX")`.
    - `XXXXXXX` is an index for the ICE.
    - An ICE index is a 7-chars hexadecimal number. It should be unique.
  - All the `panic!`s, `unreachable!`s, `assert!`, and similar stuffs shell have their own unique ICE index.
  - It's okay to panic without any index in tests.