use std::collections::HashSet;

#[test]
fn test_parse_use() {}

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
            "use A.{B, C, D.{E, F, G}};",
            vec![
                "use A.B as B;",
                "use A.C as C;",
                "use A.D.E as E;",
                "use A.D.F as F;",
                "use A.D.G as G;",
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

fn invalid() -> Vec<String> {
    vec![
        "use A.{B, C} as D;",
    ].into_iter().map(
        |s| s.to_string()
    ).collect()
}