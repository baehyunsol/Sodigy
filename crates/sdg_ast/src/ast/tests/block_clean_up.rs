use super::check_ast_of_tester;

fn samples() -> Vec<(Vec<u8>, String)> {
    vec![
        (
            "
            use foo, bar;
            def tester: Int = {
                let a = foo;
                let b = bar;
                let c = 1_3076_7436_8000;
                let d = 3000;
                a + b + c + c
            };",
            "{c=1307674368000;Add(Add(Add(foo,bar),c),c)}",
        ),
        (
            "
            use foo;
            def tester: Int = {
                let a = 3;
                let b = 4;
                foo
            };",
            "foo",
        ),
        (
            "
            use foo;
            def tester: Int = {
                let a = {
                    let a = 3;
                    let b = 4;
                    a + b
                };
                let b = {
                    let b = foo() + 5;
                    let c = foo();
                    b + c + c
                };
                a + b
            };",
            "Add(Add(3,4),{c=Call(foo);Add(Add(Add(Call(foo),5),c),c)})",
        ),
        (
            "
            use foo, bar, baz, foz, ciz, sol;
            def tester: Int = {
                let a = foo; let b = bar;
                {
                    let b = baz; let c = foz;
                    {
                        let c = ciz; let d = sol;
                        a + b + c + d
                    }
                }
            };",
            "Add(Add(Add(foo,baz),ciz),sol)",
        ),
        (
            "
            use foo, bar, baz, foz, ciz, sol;
            def tester: Int = {
                let a = foo; let b = bar;
                b + {
                    let b = baz; let c = foz;
                    c + {
                        let c = ciz; let d = sol;
                        a + b + c + d
                    }
                }
            };",
            "Add(bar,Add(foz,Add(Add(Add(foo,baz),ciz),sol)))",
        ),
    ].into_iter().map(
        |(s1, s2)| (s1.as_bytes().to_vec(), s2.to_string())
    ).collect()
}

#[test]
fn block_clean_up_test() {
    check_ast_of_tester(samples());
}
