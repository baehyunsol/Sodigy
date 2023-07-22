use crate::{LocalParseSession, parse_file};

fn valid_samples() -> Vec<(String, Vec<(String, Vec<String>)>)> {
    let result = vec![
        (
            "
@test.eq(3)
def add_test: Int = 1 + 2;",
            vec![("test.eq", vec!["3"])],
        ),
        (
            "
# I don't care about name errors...
use a.b.c;
use b;

@c.d.e
@b.c.d
@b
def foo: Int = 0;",
            vec![("a.b.c.d.e", vec![]), ("b.c.d", vec![]), ("b", vec![])],
        ),
        (
            "
@test.neq(4)
@test.eq(3)
def add_test: Int = 1 + 2;",
            vec![("test.eq", vec!["3"]), ("test.neq", vec!["4"])],
        ),
        (
            "
@test.eq([1, 2, 3])
def list_test: List(Int) = {
    let one = 1;
    let two = one + one;
    let three = one + two;

    [one, two, three]
};",
            vec![("test.eq", vec!["[1,2,3]"])],
        ),
    ];

    result.into_iter().map(
        |(sample, decorators)| (
            sample.to_string(),
            decorators.into_iter().map(
                |(name, args)| (
                    name.to_string(),
                    args.into_iter().map(
                        |arg| arg.to_string()
                    ).collect()
                )
            ).collect()
        )
    ).collect()
}

#[test]
fn valid_decorator_tests() {
    let mut session = LocalParseSession::new();

    for (sample, decorators) in valid_samples().into_iter() {
        session.set_input(sample.as_bytes().to_vec());

        match parse_file(sample.as_bytes(), &mut session) {
            Ok(ast) => {
                let actual_decos = &ast.defs.values().next().unwrap().decorators;
                assert_eq!(actual_decos.len(), decorators.len());

                let actual_deco_names = actual_decos.iter().map(
                    |deco| deco.names.iter().map(
                        |name| name.to_string(&session)
                    ).collect::<Vec<String>>().join(".")
                ).collect::<Vec<String>>();

                let actual_deco_args = actual_decos.iter().map(
                    |deco| deco.args.iter().map(
                        |arg| arg.to_string(&session)
                    ).collect::<Vec<String>>()
                ).collect::<Vec<Vec<String>>>().concat();

                for (deco_name, deco_args) in decorators.iter() {

                    if !actual_deco_names.contains(deco_name) {
                        panic!("{deco_name} not in {actual_deco_names:?}");
                    }

                    for deco_arg in deco_args.iter() {

                        if !actual_deco_args.contains(deco_arg) {
                            panic!("{deco_arg} not in {actual_deco_args:?}");
                        }

                    }

                }

            },
            Err(e) => {
                panic!("sample: {sample}\nerror: {}", e.render_err(&session));
            }
        }

    }

}