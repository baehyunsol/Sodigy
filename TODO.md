- Parsers of block_expr and lambda_def rely on the fact that commas and semi-colons do not appear in expressions. They only appear inside `{}`s, `[]`s or `()`s. -> How do I guarantee that using code?

- Make multiple crates
  - current crate only parses a file
    - doesn't care about other files
    - returns `Vec<Stmt>`
    - it also does name-resolving