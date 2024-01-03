rm -f a.out
rm -f *.hir
rm -f *.tokens
rm -f __*.tmp

# TODO: use `sodigy --clean` when it's implemented
rm -f -r __sdg_cache
rm -f -r ./samples/__sdg_cache
rm -f -r ./samples/tests/__sdg_cache

rm -f -r __tmp_*