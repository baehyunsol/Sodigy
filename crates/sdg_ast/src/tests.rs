use crate::*;

fn samples() -> Vec<(Vec<String>, Vec<u8>)> {  // (substring of the error message, input)
    vec![
        (
            vec!["try `def_`"],
            "def def;",
        ),
        (
            vec![
                "identifier `a` is bound more than once in a pattern name binding list",
            ],
            "use Foo;
            def matcher(x: Foo): Int = match x {
                ($a, $a) => 0,
                _ => 1,
            };"
        ),
        (
            vec!["the name `x` is defined more than once"],
            "use x; def x: Int = 3;",
        ),
    ].into_iter().map(
        |(error_messages, input)| (
            error_messages.into_iter().map(
                |error_message| error_message.to_string()
            ).collect(),
            input.as_bytes().to_vec()
        )
    ).collect()
}

#[test]
fn error_test() {
    let mut session = LocalParseSession::new();
    let mut failures = vec![];

    for (error_messages, input) in samples().into_iter() {
        session.set_direct_input(input.clone());

        match parse_file(&input, &mut session) {
            Ok(_) => {
                failures.push(format!(
                    "{} is supposed to fail, but it doesn't!",
                    String::from_utf8_lossy(&input),
                ));
            },
            Err(_) => {
                let e = session.render_err();

                for error_message in error_messages.iter() {
                    if !e.contains(error_message) {
                        failures.push(format!(
                            "substring {error_message:?} is not included in the error message below\n{e}"
                        ));
                    }
                }
            },
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n\n"))
    }
}
