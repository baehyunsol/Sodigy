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
        (
            "def tester: Int = {
                let adder = \\{x, \\{y, x + y}};
                let adder1 = adder(1);
                let adder2 = adder(2);
              
                adder1(20) + adder2(20)
            };",
            // TODO: hash value is very prone to change
            "Add(Call(Call(@@LAMBDA__82d10d42beeeece8ffce7fa2,1),20),Call(Call(@@LAMBDA__82d10d42beeeece8ffce7fa2,2),20))",
        ),
        (
            "
            use foo, bar;
            def tester: Int = {
                let x = a + a + b + b;
                let a = foo();
                let b = bar();

                a
            };",
            "Call(foo)"
        ),
        (
            "def tester: Int = {
                let fibo = \\{n, if n < 2 { 0 } else { fibo(n - 1) + fibo(n - 2) }};

                fibo(20)
            };",
            "TODO",
        ),
        (
            "def tester(n: Int): Int = {
                let f1 = \\{x, f2(x - 1)};
                let f2 = \\{x, if x > 0 { f1(x - n) } else { 0 } };

                f1(100)
            };",
            "TODO",
        ),
    ].into_iter().map(
        |(s1, s2)| (s1.as_bytes().to_vec(), s2.to_string())
    ).collect()
}

#[test]
fn block_clean_up_test() {
    check_ast_of_tester(samples());
}
