// This file is Very Very experimental
// json -> sodigy transformation
// 1. `sodigy_lex` and `sodigy_parse` can digest json files
// 2. with very small transformations, we can convert a json file to a sodigy file (in Vec<TokenTree> level)
// 3. the converted file goes through normal compilation process
//
// The benefits are
// 1. it emits sodigy-style errors and warnings
// 2. less rust code, and more sodigy code
//
// It can only parse `sodigy.json`, not all json files. `sodigy.json` looks like below
// ```
// {
//     "macros": {
//         "foo": "path/to/foo"
//     },
//     "dependencies": {
//         "bar": "path/to/bar",
//         "baz": "path/to/baz",
//     },
// }
// ```
// The above code is converted to below
// ```
// let config = SodigyConfig.base() `macros [("foo", "path/to/foo")] `dependencies [("bar", "path/to/bar"), ("baz", "path/to/baz")];
// ```
// Since it uses sodigy's parser, the syntax is a bit more loose than the original json's.
// For example, it allows comments and trailing commas.

use crate::{Token, TokenKind};
use crate::error::AstError;

fn json_to_sodigy(tokens: &Vec<Token>) -> Result<Vec<Token>, Vec<AstError>> {
    match tokens.get(0) {
        Some(Token {
            kind: TokenKind::Group {
                delim: Delim::Brace,
                tokens,
                prefix: b'\0',
            },
            ..
        }) => {
            // parse its content
            todo!()
        },
        Some(token) => Err(vec![
            AstError::unexpected_token(token.clone()),
        ]),
        None => Err(vec![
            AstError::unexpected_end(),
        ]),
    }
}
