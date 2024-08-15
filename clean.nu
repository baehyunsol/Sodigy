rm -f *.hir
rm -f *.mir

# this line must be the last line because
# `cargo run` might fail
cargo run -- --clean
