# Internal Compiler Errors

Compiler should not panic in any case. If it panics, that's an error.

- Rules for `unwrap`, `expect`, and `panic!`
  - `unwrap` should be avoided in any case.
  - If you want to unwrap something, use `expect("Internal Compiler Error XXXXXXX")`.
    - `XXXXXXX` is an index for the ICE.
    - An ICE index is a 7-chars hexadecimal number. It should be unique.
  - All the `panic!`s, `unreachable!`s, `assert!`, and similar stuffs shell have their own unique ICE index.
  - It's okay to panic without any index in tests.

# External Compiler Errors

# Compiler Warnings

Unused names, always true (in branch)

# Rust

Most error messages and warning messages are from Rust.

- Warnings
  - ``` unused variable: `b` ```
  - ``` unused import: `std::fs::File` ```
  - ``` methods `is_identifier` and `get_first_token` are never used ```
  - ``` fields `kind` and `given_path` are never read ```
  - ``` variants `FileNotFound`, `PermissionDenied`, and `AlreadyExists` are never constructed ```
  - ``` associated function `init` is never used ```
  - ``` function `read_bytes` is never used ```
  - ``` unreachable statement ```
- Errors
  - ``` cannot find function `doto` in this scope ```
  - ``` cannot find macro `printll` in this scope ```
  - ``` no method named `write` found for struct `File` in the current scope ```
    - ``` help: a macro with a similar name exists: `println` ```
  - ``` mismatched types ```
    - ``` expected `Result<AST, Box<dyn SodigyError>>`, found `bool` ```
  - ``` `Result<String, FromUtf8Error>` doesn't implement `std::fmt::Display` ```
    - ``` `Result<String, FromUtf8Error>` cannot be formatted with the default formatter ```
  - ``` couldn't read test.rs: stream did not contain valid UTF-8 ```
  - ``` non-item in item list ```
    - when I placed `[]` after a method definition, for no reason
  - ``` unresolved import `crate::err::SodigyError` ```
    - ``` no `SodigyError` in `err` ```
  - ``` expected item after attributes ```
  - ``` expected one of `(`, `,`, `=`, `{`, or `}`, found `:` ```
  - ``` the name `ASTErrorKind` is defined multiple times ```
  - ``` this file contains an unclosed delimiter ```
  - ``` unterminated double quote string ```
  - ``` identifier `a` is bound more than once in this parameter list ```