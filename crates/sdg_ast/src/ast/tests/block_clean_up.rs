use super::check_ast_of_tester;

fn samples() -> Vec<(Vec<u8>, String)> {
    vec![
        (
            "
            use foo, bar;
            def tester: Int = {
                a = foo;
                b = bar;
                c = 1_3076_7436_8000;
                d = 3000;
                a + b + c + c
            };",
            "{c=1307674368000;Add(Add(Add(foo,bar),c),c)}",
        ),
        (
            "
            use foo;
            def tester: Int = {
                a = 3;
                b = 4;
                foo
            };",
            "foo",
        ),
        (
            "
            use foo;
            def tester: Int = {
                a = {
                    a = 3;
                    b = 4;
                    a + b
                };
                b = {
                    b = foo() + 5;
                    c = foo();
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
                a = foo; b = bar;
                {
                    b = baz; c = foz;
                    {
                        c = ciz; d = sol;
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
                a = foo; b = bar;
                b + {
                    b = baz; c = foz;
                    c + {
                        c = ciz; d = sol;
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
