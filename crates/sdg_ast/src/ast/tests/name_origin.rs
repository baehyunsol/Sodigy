use super::super::NameOrigin;
use crate::{LocalParseSession, parse_file};
use crate::stmt::LAMBDA_FUNC_PREFIX;
use crate::utils::bytes_to_string;

fn samples() -> Vec<Vec<u8>> {
    vec![
        "use global__.sub__.sub__;

        def local___: Int = 3;

        @sub__(local___)
        def local__<generic__>(func__: List(Int), func___: List(Int), block_scoped: Int): Int = {
            let block__ = 3;
            let block___ = 4;
            let block_scoped = 5;
            let block______ = sub__;
            let block_______ = \\{func__, func___, func__ + func___};
            let block_________ = match sub__ {
                ($match__, $match___) => match__ + match___,
                _ => 1,
            };

            func__[local___] + func___[local___ + 1] + block__ + block___ + block_scoped + block______ + block_______(block__) + block_________
        };",
    ].into_iter().map(|s| s.as_bytes().to_vec()).collect()
}

#[test]
fn name_origin_test() {
    let mut session = LocalParseSession::new();

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

                    // TODO: add case for this
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
}
