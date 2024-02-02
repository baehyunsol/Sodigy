rm -f *.out
rm -f *.hir
rm -f *.tokens
rm -f __*.tmp
cargo run -- --clean
rm -f -r __tmp_*
