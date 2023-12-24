# Experimental

let start = date now | date to-record
let start_sec = $start.second + $start.minute * 60 + $start.hour * 3600 + $start.day * 86400

cargo clean
cd crates/sodigy_ast
cargo test
cargo test --release
cd ../sodigy_clap
cargo test
cargo test --release
cd ../sodigy_endec
cargo test
cargo test --release
cd ../sodigy_error
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
cargo doc
cargo test
cargo test --release

# it requires `cargo-depgraph`
cargo depgraph | dot -Tpng | save -f dep_graph.png

let end = date now | date to-record
let end_sec = $end.second + $end.minute * 60 + $end.hour * 3600 + $end.day * 86400

let elt = $end_sec - $start_sec

echo $"took ($elt) seconds..."
