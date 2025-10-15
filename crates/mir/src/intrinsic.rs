// `Intrinsic`s aren't always directly mapped to Sodigy functions. For example,
// Sodigy's `panic` calls `Intrinsic::Eprint` then `Intrinsic::Panic`.

#[derive(Clone, Copy, Debug)]
pub enum Intrinsic {
    // pure
    // `Fn(Int, Int) -> Int`
    // The compiler assumes that there's no integer overflow.
    IntegerAdd,

    // pure
    // `Fn(Int, Int) -> Int`
    // The compiler assumes that there's no integer overflow.
    IntegerSub,

    // pure
    // `Fn(Int, Int) -> Int`
    // The compiler assumes that there's no integer overflow.
    IntegerMul,

    // pure
    // `Fn(Int, Int) -> Int`
    // If divisor is 0, it's UB. Sodigy code must make sure that divisor is not 0.
    IntegerDiv,

    // pure
    // `Fn(Int, Int) -> Bool`
    IntegerEq,

    // pure
    // `Fn(Int, Int) -> Bool`
    IntegerGt,

    // pure
    // `Fn(Int, Int) -> Bool`
    IntegerLt,

    // impure
    // `Fn()`
    // Immediately terminates the program with exit code 1.
    Panic,

    // impure
    // `Fn()`
    // Immediately terminates the program with exit code 0.
    Exit,

    // impure
    // `Fn(String)`
    // It prints the string to stdout.
    Print,

    // impure
    // `Fn(String)`
    // It prints the string to stderr.
    EPrint,
}
