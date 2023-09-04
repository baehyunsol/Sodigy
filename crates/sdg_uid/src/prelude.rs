use super::{PRELUDE, UID, ZERO};

// UIDs of preludes and builtins
// For now, I don't have any rules for allocating UIDs.
// All I care is that they're unique and doesn't change across compilations.
// They may change as the compiler changes, though.

macro_rules! def_prelude_uid {
    ($name: ident, $id: literal) => {
        pub const fn $name() -> UID {
            UID(($id << 32) & ZERO | PRELUDE)
        }
    };
}

// type of all type `def Type: Type;  # Built in`
// e.g. `Option.is_subtype_of(Type) == Bool.True`
def_prelude_uid!(type_, 0x0001);

// int type `def Int: Type;  # Built in`
def_prelude_uid!(int, 0x0002);

// list type `def List(t: Type): Type;  # Built in`
def_prelude_uid!(list, 0x0003);

// function type `def Func(##! types of args and ret_val !##): Type;  # Built in`
def_prelude_uid!(func, 0x0004);

// string type `def String: Type = List(Char)`
def_prelude_uid!(string, 0x0005);

// boolean type `enum Bool { True, False }`
def_prelude_uid!(bool, 0x0006);

// option type `enum Option<T> { None, Some(T) }`
def_prelude_uid!(option, 0x0007);

// test decorator (builtin)
def_prelude_uid!(test_, 0x0008);

// number type `struct Number { denom: Int, numer: Int }`
def_prelude_uid!(number, 0x0009);

// result type `enum Result<T, E> { Ok(T), Err(E) }`
def_prelude_uid!(result, 0x000a);

// char type `def Char: Type = Int;`
def_prelude_uid!(char, 0x000b);

// bytes type
// TODO: how do I define it?
def_prelude_uid!(bytes, 0x000c);
