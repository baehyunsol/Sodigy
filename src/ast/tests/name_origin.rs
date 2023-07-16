use super::super::NameOrigin;
use crate::{LocalParseSession, parse_file};
use crate::utils::bytes_to_string;

fn samples() -> Vec<Vec<u8>> {
    vec![
        b"use global__.sub__.sub__;

        def local___: Int = 3;

        @sub__(local___)
        def local__(func__: List(Int), func___: List(Int)): Int = {
            block__ = 3;
            block___ = 4;
            func__[local___] + func___[local___ + 1] + block__ + block___
        };",
    ].into_iter().map(|s| s.to_vec()).collect()
}

#[test]
fn name_origin_test() {
    std::env::set_var("RUST_BACKTRACE", "FULL");
    let mut session = LocalParseSession::new();

    for sample in samples() {
        session.set_input(sample.clone());
        let ast = match parse_file(&sample, &mut session) {
            Ok(a) => a,
            Err(e) => {
                panic!("{}\n\n{}", bytes_to_string(&sample), e.render_err(&session))
            },
        };

        ast.id_walker(
            |name, origin| {
                let name = session.unintern_string(*name);
                println!("{:?}", bytes_to_string(&name));

                match origin {
                    NameOrigin::Global => assert!(name.starts_with(b"global")),
                    NameOrigin::SubPath => assert!(name.starts_with(b"sub")),
                    NameOrigin::Local => assert!(name.starts_with(b"local")),
                    NameOrigin::FuncArg => assert!(name.starts_with(b"func")),
                    NameOrigin::BlockDef(_) => assert!(name.starts_with(b"block")),
                    NameOrigin::Prelude => {},
                    NameOrigin::NotKnownYet => panic!("{}", bytes_to_string(&name)),
                }
            }
        );
    }
}
