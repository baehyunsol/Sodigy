use super::super::NameOrigin;
use crate::ast::Opt;
use crate::parse_file;
use crate::session::LocalParseSession;
use crate::stmt::LAMBDA_FUNC_PREFIX;
use crate::utils::bytes_to_string;

fn samples() -> Vec<Vec<u8>> {
    vec![
        "use global_2.sub_2.sub_2;

        def local_3: Int = 3;

        @sub_2(local_3)
        def local_2<generic_2>(func_2: List(Int), func_3: List(Int), block_scoped: Int): Int = {
            let block_2 = 3;
            let block_3 = 4;
            let block_scoped = 5;
            let block_6 = sub_2;
            let block_7 = \\{func_2, func_3, func_2 + func_3};
            let block_8 = match sub_2 {
                ($match_2, $match_3) => match_2 + match_3,
                _ => 1,
            };

            func_2[local_3] + func_3[local_3 + 1] + block_2 + block_3 + block_scoped + block_6 + block_7(block_2) + block_8
        };",
        "
        use local_0.sub_0.sub_0 as local_1;
        use global_3;
        use global_type;

        def local_0: global_type = global_3();
        def local_2: Int = local_1 + local_1;
        ",
    ].into_iter().map(|s| s.as_bytes().to_vec()).collect()
}

#[test]
fn name_origin_test() {
    let mut session = LocalParseSession::new();
    session.toggle(Opt::IntraInterMod, false);

    for sample in samples() {
        session.set_direct_input(sample.clone());
        let ast = match parse_file(&sample, &mut session) {
            Ok(a) => a,
            Err(_) => {
                panic!("{}\n\n{}", bytes_to_string(&sample), session.render_err())
            },
        };

        ast.id_walker(
            |name, origin, _| {
                let name = session.unintern_string(*name);

                match origin {
                    NameOrigin::Global => assert_prefix(&name, b"global"),
                    NameOrigin::SubPath => assert_prefix(&name, b"sub"),
                    NameOrigin::Local => assert_prefix(&name, b"local"),
                    NameOrigin::FuncArg(_) => assert_prefix(&name, b"func"),
                    NameOrigin::GenericArg(_) => assert_prefix(&name, b"generic"),
                    NameOrigin::BlockDef(_) => assert_prefix(&name, b"block"),
                    NameOrigin::Prelude => {},
                    NameOrigin::NotKnownYet => panic!("{}", bytes_to_string(&name)),
                    NameOrigin::AnonymousFunc => assert_prefix(&name, LAMBDA_FUNC_PREFIX.as_bytes()),
                    NameOrigin::MatchBranch(_, _) => assert_prefix(&name, b"match"),
                }
            },
            &mut ()
        );
    }
}

fn assert_prefix(n: &[u8], prefix: &[u8]) {
    if !n.starts_with(prefix) {
        let n = bytes_to_string(n);
        let prefix = bytes_to_string(prefix);

        panic!("{n:?} doesn't start with {prefix:?}");
    }

    let n = bytes_to_string(n);
    let prefix = bytes_to_string(prefix);
    println!("{n:?} does start with {prefix:?}");
}
