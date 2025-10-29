1. lex -> parse -> hir (per file)
  - It may depend on external files, if there's a custom macro.
  - The compiler can create a per-file hir cache for incremental compilation.
2. inter-file hir analysis
  - Once hir of all files are generated, it creates a giant map.
  - Each hir does its own analysis using the map.
3. mir
  - It's per-file: it requires hir of the file, and the giant map from step 2.
4. inter-file mir analysis
5. 
