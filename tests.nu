# Experimental

cargo clean
cd crates/sodigy_ast
cargo test
cargo test --release
cd ../sodigy_endec
cargo test
cargo test --release
cd ../sodigy_err
cargo test
cargo test --release
cd ../sodigy_files
cargo test
cargo test --release
cd ../sodigy_high_ir
cargo test
cargo test --release
cd ../sodigy_intern
cargo test
cargo test --release
cd ../sodigy_interpreter
cargo test
cargo test --release
cd ../sodigy_keyword
cargo test
cargo test --release
cd ../sodigy_lex
cargo test
cargo test --release
cd ../sodigy_number
cargo test
cargo test --release
cd ../sodigy_parse
cargo test
cargo test --release
cd ../sodigy_prelude
cargo test
cargo test --release
cd ../sodigy_span
cargo test
cargo test --release
cd ../sodigy_test
cargo test
cargo test --release
cd ../sodigy_uid
cargo test
cargo test --release
cd ../..
cargo test
cargo test --release
