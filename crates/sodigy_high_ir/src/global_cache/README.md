# Hir Global Cache

When there are multiple files to compile, the HIR sessions are built in parallel.

The global cache is the control tower of the parallel process. It tells workers which file to read. When the workers finished constructing an HIR session, the session is pushed to the global cache. The compiler later reads the result in the cache to build MIR.
