# Experimental

let start = date now | date to-record
let start_sec = $start.second + $start.minute * 60 + $start.hour * 3600 + $start.day * 86400

cargo clean

let crates = [
    "ast", "clap", "endec",
    "error", "files", "high_ir",
    "intern", "keyword",
    "lex", "mid_ir",
    "number", "parse", "prelude",
    "span", "test", "uid",
]

for $crate in $crates {
    cd $"./crates/sodigy_($crate)"
    cargo test
    cargo test --release
    cd ../..
}

cargo doc
cargo test
cargo test --release

# it requires `cargo-depgraph`
cargo depgraph | dot -Tpng | save -f dep_graph.png

nu clean.nu

let end = date now | date to-record
let end_sec = $end.second + $end.minute * 60 + $end.hour * 3600 + $end.day * 86400

let elt = $end_sec - $start_sec

echo $"took ($elt) seconds..."
