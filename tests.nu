def main [--depgraph] {
    let start = date now | date to-record
    let start_sec = $start.second + $start.minute * 60 + $start.hour * 3600 + $start.day * 86400

    $env.RUST_LOG = "trace"
    cargo clean

    cargo doc
    cargo test
    cargo test --release

    let crates = [
        "ast", "clap", "config",
        "collect_names", "endec",
        "error", "files", "high_ir",
        "intern", "keyword",
        "lex", "mid_ir", "number",
        "output", "parse", "prelude",
        "session", "span", "uid",
    ]

    for $crate in $crates {
        cd $"./crates/sodigy_($crate)"
        cargo test
        cargo test --release
        cd ../..
    }

    if $depgraph {
        cargo depgraph | dot -Tpng | save -f dep_graph.png
    }

    nu clean.nu
    nu link_bin.nu

    chmod +x ./sodigy

    # TODO: run `./sodigy --test XXX.sdg` here

    $env.RUST_LOG = "off"

    let end = date now | date to-record
    let end_sec = $end.second + $end.minute * 60 + $end.hour * 3600 + $end.day * 86400

    let elt = $end_sec - $start_sec

    echo $"Complete! It took ($elt) seconds..."
}
