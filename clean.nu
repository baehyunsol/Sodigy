rm -f *.hir
rm -f *.mir
rm -f __*.tmp
rm -f -r __tmp_*

# this line must be the last line because
# `cargo run` might fail
cargo run -- --clean
