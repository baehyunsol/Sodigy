// A parsed token is always an unsigned number because `-` is parsed to a unary minus operator.
// Still, `InternedNumber` is signed because some internal operations subtract `InternedNumber`s.
//
// A numeric literal "1.0" becomes `SmallInteger(1)`, not `SmallRatio(1_000_000)` because the goal
// of `InternedNumber` is to intern the value, not the literal. It's lexer's responsibility to
// remember whether it's a float literal or an integer literal.
pub enum InternedNumber {
    SmallInteger(i64),

    // Fixed point representation of the number (n = number * 1_000_000).
    // The number has to be representable in this format (e.g. its fractional part has less than 7 digits).
    SmallRatio(i64),

    BigInteger(BigInt),
    BigRatio(Ratio),
}

impl InternedNumber {
    /// It assumes that the input is valid. It's lexer's responsibility to guarantee that.
    pub fn parse_integer(s: &[u8]) -> Self {
        let s = String::from_utf8(s).unwrap();

        match s.parse::<i64>() {
            Ok(n) => InternedNumber::SmallInteger(n),
            Err(_) => InternedNumber::BigInteger(BigInt::from_str(s).unwrap()),
        }
    }
}
