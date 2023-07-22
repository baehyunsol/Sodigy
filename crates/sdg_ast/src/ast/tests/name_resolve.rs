use super::check_ast_of_tester;

fn samples() -> Vec<(Vec<u8>, String)> {
    vec![
        (
            "use a.b.c as d;
            def tester: Int = d.b.c;",
            "Path(Path(Path(Path(a,b),c),b),c)",
        ), (
            "use a.b.c as d;
            use a;
            def tester: Int = a.d.b;",
            "Path(Path(a,d),b)",
        ), (
            "use a.b.c;
            def tester: Int = c.c.c;",
            "Path(Path(Path(Path(a,b),c),c),c)",
        ), (
            "use b as c;
            use b;
            def tester: Int = b.b.c;",
            "Path(Path(b,b),c)",
        ), (
            "use c as b;
            def tester: Int = b.b.c;",
            "Path(Path(c,b),c)",
        ), (
            "use a.b as c;
            use b;
            def tester: Int = b.b.c;",
            "Path(Path(b,b),c)",
        ), (
            "use a.b as c;
            def tester: Int = c.b.c;",
            "Path(Path(Path(a,b),b),c)",
        ), (
            "use a.b.c.d.e.f.g.h.i.j as c;
            use b;
            def tester: Int = b.b.c;",
            "Path(Path(b,b),c)",
        ), (
            "use a.b.c.d.e.f.g.h.i.j as c;
            def tester: Int = c.b.c;",
            "Path(Path(Path(Path(Path(Path(Path(Path(Path(Path(Path(a,b),c),d),e),f),g),h),i),j),b),c)",
        ), (
            "use a.b.c; use foo; use bar;
            def tester: Int = {let c = foo; let a = bar; a + c};",
            "Add(bar,foo)",
        ),
    ].into_iter().map(
        |(s1, s2)| (s1.as_bytes().to_vec(), s2.to_string())
    ).collect()
}

#[test]
fn name_resolve_test() {
    check_ast_of_tester(samples());
}
