use super::{ENUM, FUNC, STRUCT, UID, ZERO};

// UIDs of preludes and builtins
// For now, I don't have any rules for allocating UIDs.
// All I care is that they're unique and doesn't change across compilations.
// They may change as the compiler changes, though.

// type of every type (builtin)
// e.g. `Option.is_subtype_of(Type) == Bool.True`
pub fn type_() -> UID {
    UID((0x0001 << 32) & ZERO | ENUM)
}

// int type (builtin)
pub fn int() -> UID {
    UID((0x0002 << 32) & ZERO | ENUM)
}

// list type (builtin)
pub fn list() -> UID {
    UID((0x0003 << 32) & ZERO | FUNC)  // generic types are functions
}

// function type (builtin)
pub fn func() -> UID {
    UID((0x0004 << 32) & ZERO | FUNC)  // generic types are functions
}

// string type `def String: Type = List(Char)`
pub fn string() -> UID {
    UID((0x0005 << 32) & ZERO | FUNC)
}

// boolean type `enum Bool { True, False }`
pub fn bool() -> UID {
    UID((0x0006 << 32) & ZERO | ENUM)
}

// option type `enum Option<T> { None, Some(T) }`
pub fn option() -> UID {
    UID((0x0007 << 32) & ZERO | ENUM)
}

// test decorator (builtin)
pub fn test_() -> UID {
    UID((0x0008 << 32) & ZERO | FUNC)  // TODO: is it FUNC?
}

// number type `struct Number { denom: Int, numer: Int }`
pub fn number() -> UID {
    UID((0x0009 << 32) & ZERO | STRUCT)
}

// result type `enum Result<T, E> { Ok(T), Err(E) }`
pub fn result() -> UID {
    UID((0x000a << 32) & ZERO | ENUM)
}

// char type `def Char: Type = Int;`
pub fn char() -> UID {
    UID((0x000b << 32) & ZERO | FUNC)  // TODO: is it FUNC?
}

// bytes type
// TODO: how do I define it?
pub fn bytes() -> UID {
    UID((0x000c << 32) & ZERO | FUNC)  // TODO: is it FUNC?
}
