1. lex -> parse -> hir (per file)
  - It may depend on external files, if there's a custom macro.
  - The compiler can create a per-file hir cache for incremental compilation.
2. inter-file hir analysis
3. mir
  - It's per-file, but requires the result of inter-file hir analysis.
4. inter-file mir analysis
5. 
