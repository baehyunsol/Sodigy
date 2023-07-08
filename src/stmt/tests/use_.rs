use crate::err::SodigyError;
use crate::lexer::lex_tokens;
use crate::session::LocalParseSession;
use crate::span::Span;
use crate::stmt::{parse_use, Use};
use crate::token::TokenList;
use std::collections::HashSet;

impl Use {

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        format!("use {} as {};", self.path.to_string(session), self.alias.to_string(session))
    }

}

#[test]
fn test_parse_use() {
    let mut session = LocalParseSession::new();

    for (sample, desired) in sample().into_iter() {
        let input = sample.as_bytes().to_vec();
        session.set_input(input.clone());
        let tokens = match lex_tokens(&input, &mut session) {
            Ok(t) => t,
            Err(e) => {
                panic!("ParseError at `lex_tokens`! sample: {sample:?}, desired: {desired:?}\n\n{}", e.render_err(&session));
            }
        };
        let mut tokens = TokenList::from_vec(tokens);

        // skip `use`
        tokens.step().expect("Internal Compiler Error B01EA26");

        let uses = match parse_use(&mut tokens, Span::first(), true) {
            Ok(u) => u,
            Err(e) => {
                panic!("ParseError at `parse_use`! sample: {sample:?}, desired: {desired:?}\n\n{}", e.render_err(&session));
            }
        };

        let result = uses.iter().map(|u| u.to_string(&session)).collect::<HashSet<String>>();

        assert_eq!(result, desired);
    }

    for (sample, span) in invalid().into_iter() {
        let input = sample.as_bytes().to_vec();
        session.set_input(input.clone());
        let tokens = match lex_tokens(&input, &mut session) {
            Ok(t) => t,
            Err(e) => {
                panic!("ParseError at `lex_tokens`! sample: {sample:?}\n\n{}", e.render_err(&session));
            }
        };
        let mut tokens = TokenList::from_vec(tokens);

        // skip `use`
        tokens.step().expect("Internal Compiler Error B01EA26");

        match parse_use(&mut tokens, Span::first(), true) {
            Ok(u) => {
                panic!(
                    "sample: {sample:?} is supposed to panic, but returns {:?}",
                    u.iter().map(|u| u.to_string(&session)).collect::<Vec<String>>()
                );
            }
            Err(e) => {
                if e.span.index != span {
                    panic!("desired span: {span}\n\nactual error: {}", e.render_err(&session));
                }
            }
        }

    }
}

/*
 * `use A.B;` -> `use A.B as B;`
 * `use A.B.C;` -> `use A.B.C as C;`
 * `use A.B, C.D;` -> `use A.B; use C.D;`
 * `use {A.B, C.D};` -> `use A.B; use C.D;`
 * `use A.{B, C, D};` -> `use A.B; use A.C; use A.D;`
 * `use A.B, C, D;` -> `use A.B; use C; use D;`
 * `use A.{B as C, D as E};` -> `use A.B as C; use A.D as E;`
 * `use A.{B, C} as D;` -> Invalid
 */
fn sample() -> Vec<(String, HashSet<String>)> {
    vec![
        (
            "use A.B;",
            vec![
                "use A.B as B;",
            ],
        ),
        (
            "use A.B.C;",
            vec![
                "use A.B.C as C;",
            ],
        ),
        (
            "use A.B.C as C;",
            vec![
                "use A.B.C as C;",
            ],
        ),
        (
            "use A.B, C.D;",
            vec![
                "use A.B as B;",
                "use C.D as D;",
            ],
        ),
        (
            "use {A.B, C.D};",
            vec![
                "use A.B as B;",
                "use C.D as D;",
            ],
        ),
        (
            "use A.{B, C, D};",
            vec![
                "use A.B as B;",
                "use A.C as C;",
                "use A.D as D;",
            ],
        ),
        (
            "use A.{B, C, D}, E.F;",
            vec![
                "use A.B as B;",
                "use A.C as C;",
                "use A.D as D;",
                "use E.F as F;",
            ],
        ),
        (
            "use A.{B, C, D,}, E.F;",
            vec![
                "use A.B as B;",
                "use A.C as C;",
                "use A.D as D;",
                "use E.F as F;",
            ],
        ),
        (
            "use A.{B, C, D.{E, F, G}, H};",
            vec![
                "use A.B as B;",
                "use A.C as C;",
                "use A.D.E as E;",
                "use A.D.F as F;",
                "use A.D.G as G;",
                "use A.H as H;",
            ],
        ),
        (
            "use A.B, C, D;",
            vec![
                "use A.B as B;",
                "use C as C;",
                "use D as D;",
            ],
        ),
        (
            "use A.{B as C, D as E};",
            vec![
                "use A.B as C;",
                "use A.D as E;",
            ],
        ),
    ].into_iter().map(
        |(before, after)| (
            before.to_string(),
            after.into_iter().map(|s| s.to_string()).collect::<HashSet<String>>()
        )
    ).collect()
}

fn invalid() -> Vec<(String, usize)> {
    vec![
        ("use A.{B, C} as D;", 13),
        ("use A.{};", 6),
        ("use A.{,};", 7),
        ("use A.{B, C;};", 11),
        ("use A as B as C;", 11),
    ].into_iter().map(
        |(s, ind)| (s.to_string(), ind)
    ).collect()
}
