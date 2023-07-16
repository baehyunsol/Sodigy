use crate::session::LocalParseSession;
use crate::parse_file;

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
            "use a.b.c;
            def tester: Int = {c = 3; a = 4; a + c};",
            "{c=3;a=4;Add(a,c)}",
        ),
    ].into_iter().map(
        |(s1, s2)| (s1.as_bytes().to_vec(), s2.to_string())
    ).collect()
}

#[test]
fn name_resolve_test() {
    let mut session = LocalParseSession::new();
    let test_func_name = b"tester".to_vec();

    for (sample, desired) in samples() {
        session.set_input(sample.clone());
        let ast = match parse_file(&sample, &mut session) {
            Ok(a) => a,
            Err(e) => panic!("{}", e.render_err(&session)),
        };

        assert_eq!(ast.dump_ast_of_def(test_func_name.clone(), &session).unwrap(), desired);
    }

}
