// It iterates HIR Sessions in HirGlobalCache, and ... 
// TODO: the root is not in the cache

// The result of this module contains all the Uids of all the names.
// For example, when the compiler sees `foo.bar`, it has to know what `foo` is. If it's a func,
// it needs Uid of `foo` (it must be top-level then). If it's a module, the Uid of `bar` must be
// inside its data structure.

// TODO: what if `foo` is a struct and `bar` is its field, where should the compiler look for in order to get index of `bar`?
