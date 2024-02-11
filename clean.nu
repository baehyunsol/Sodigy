rm -f *.hir
rm -f *.mir
rm -f __*.tmp
cargo run -- --clean
rm -f -r __tmp_*
