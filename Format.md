- `fmt::Display` for types
  - Outputs Sodigy code.
  - Not necessarily compilable, but is valid Sodigy code in most cases.
- `fmt::Debug` for types
  - Tries to be as informative as possible.
  - Tries to be readable.
    - For example, `&[u8]`s have to go through `String::from_utf8` before outputted.
- `render_error`
  - Error messages use this function to format internal objects.
  - In most cases, `fmt::Display` and `render_error` are the same. In that case, there's no need for extra `impl`.

Those functions are usually very expensive, because they have to access intern sessions, which requires locks.
