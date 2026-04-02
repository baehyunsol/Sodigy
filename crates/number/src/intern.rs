use crate::{
    Base,
    BigInt,
    Ratio,
    add_ubi,
    bi_to_string,
    div_bi,
    div_ubi,
    gcd_ubi,
    mul_ubi,
    powi_ubi,
    rem_ubi,
    ubi_to_string,
};
use sodigy_endec::Endec;
use sodigy_fs_api::{
    FileError,
    FileErrorKind,
    WriteMode,
    join3,
    read_bytes,
    write_bytes,
};
use sodigy_string::hash;
use std::cmp::Ordering;
use std::fmt;

// First 2 bits: type
//   - 00: SmallInt
//   - 01: SmallRatio
//   - 10: BigInt
//   - 11: BigRatio
// Next 1 bit: boolean flag (is_integer)
// Remaining 125 bits: payload
//   - SmallInt: i125 (2s complement)
//     - Its range is -21267647932558653966460912964485513216..=21267647932558653966460912964485513215
//   - SmallRatio: { numer: i63, denom: u62 }
//     - numer's range is -4611686018427387904..=4611686018427387903
//     - denom's range is 0..=4611686018427387903
//     - numer and denom are coprimes.
//   - BigInt/BigRatio: the least significant 96 bits are the hash id of the value
//     - The actual value is stored on disk.
//
// If a value can be represented in `SmallInt`, it has to be represented in `SmallInt`.
// For example, `SmallInt { n: 1 }` and `SmallRatio { numer: 1, denom: 1 }` are the
// same values, so `SmallRatio { numer: 1, denom: 1 }` should never exist.
// Similarly, if a value can be represented in `SmallRatio`, but not in `SmallInt`,
// it has to be represented in `SmallRatio`.
// The precedence is SmallInt > SmallRatio > BigInt > BigRatio.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct InternedNumber(pub u128);

const SMALL_INT: u128 = 0;
const SMALL_INT_PAYLOAD_MASK: u128 = (1 << 125) - 1;
const SMALL_RATIO: u128 = 1 << 126;
const SMALL_RATIO_NUMER_PAYLOAD_MASK: u128 = (1 << 63) - 1;
const BIG_INT: u128 = 1 << 127;
const BIG_RATIO: u128 = 3 << 126;
const IS_INTEGER: u128 = 1 << 125;

impl InternedNumber {
    /// It's not about the value, but about the original literal.
    /// For example, literal `1` and `1.0` have the same value, but the
    /// former is `is_integer`, while the latter is not `is_integer`.
    pub fn is_integer(&self) -> bool {
        self.0 & IS_INTEGER != 0
    }

    /// This is for debugging.
    pub fn dump(&self, intermediate_dir: &str) -> String {
        let n = unintern_number(*self, intermediate_dir).unwrap();

        if self.is_integer() {
            let (is_neg, nums) = div_bi(n.numer.is_neg, &n.numer.nums, n.denom.is_neg, &n.denom.nums);
            bi_to_string(is_neg, &nums)
        }

        else {
            let int_nums = div_ubi(&n.numer.nums, &n.denom.nums);

            // numer % denom * 1_000_000 / denom
            let frac = div_ubi(&mul_ubi(&rem_ubi(&n.numer.nums, &n.denom.nums), &[1_000_000]), &n.denom.nums)[0];
            let mut s = format!("{}.{frac:06}", ubi_to_string(&int_nums)).into_bytes();

            while s.len() > 3 && *s.last().unwrap() == b'0' {
                s.pop().unwrap();
            }

            let s = String::from_utf8(s).unwrap();

            if n.numer.is_neg {
                format!("-{s}")
            }

            else {
                s
            }
        }
    }
}

impl InternedNumber {
    pub fn from_u32(n: u32, is_integer: bool) -> Self {
        InternedNumber(((is_integer as u128) << 125) | n as u128)
    }

    pub fn from_i32(n: i32, is_integer: bool) -> Self {
        let n = n as i128;
        let n = n as u128 & SMALL_INT_PAYLOAD_MASK;
        InternedNumber(((is_integer as u128) << 125) | n)
    }

    #[must_use = "method returns a new number and does not mutate the original value"]
    pub fn negate(&self) -> Self {
        let is_integer = self.is_integer();

        match self.0 >> 126 {
            0 => {
                let n = interpret_small_int(self.0);

                match -n {
                    n @ -21267647932558653966460912964485513216..=21267647932558653966460912964485513215 => InternedNumber(
                        ((is_integer as u128) << 125) | (n as u128) & SMALL_INT_PAYLOAD_MASK,
                    ),
                    _ => todo!(),
                }
            },
            1 => {
                let (numer, denom) = interpret_small_ratio(self.0);

                match -numer {
                    -4611686018427387904..=4611686018427387903 => InternedNumber(
                        ((is_integer as u128) << 125) | SMALL_RATIO | (((numer as u128) & SMALL_RATIO_NUMER_PAYLOAD_MASK) << 62) | denom as u128,
                    ),
                    _ => todo!(),
                }
            },
            2 => todo!(),
            3 => todo!(),
            _ => unreachable!(),
        }
    }

    #[must_use = "method returns a new number and does not mutate the original value"]
    pub fn add_one(&self) -> Self {
        let is_integer = self.is_integer();

        match self.0 >> 126 {
            0 => {
                let n = interpret_small_int(self.0);

                match n + 1 {
                    n @ -21267647932558653966460912964485513216..=21267647932558653966460912964485513215 => InternedNumber(
                        ((is_integer as u128) << 125) | (n as u128) & SMALL_INT_PAYLOAD_MASK,
                    ),
                    _ => todo!(),
                }
            },
            _ => todo!(),
        }
    }

    pub fn cmp(self, other: InternedNumber, intermediate_dir: &str) -> Ordering {
        match (self.0 >> 126, other.0 >> 126) {
            (0, 0) => {
                let lhs = interpret_small_int(self.0);
                let rhs = interpret_small_int(other.0);
                lhs.cmp(&rhs)
            },
            _ => todo!(),
        }
    }
}

impl fmt::Debug for InternedNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let is_integer = self.is_integer();

        match self.0 >> 126 {
            0 => {
                let n = interpret_small_int(self.0);
                write!(formatter, "SmallInt {{ n: {n}, is_integer: {is_integer} }}")
            },
            1 => {
                let (numer, denom) = interpret_small_ratio(self.0);
                write!(formatter, "SmallRatio {{ numer: {numer}, denom: {denom}, is_integer: {is_integer} }}")
            },
            2 => write!(formatter, "BigInt {{ hash: {:024x} }}", self.0 & 0xffff_ffff_ffff_ffff_ffff_ffff),
            3 => write!(formatter, "BigRatio {{ hash: {:024x} }}", self.0 & 0xffff_ffff_ffff_ffff_ffff_ffff),
            _ => unreachable!(),
        }
    }
}

pub fn unintern_number(n: InternedNumber, intermediate_dir: &str) -> Result<Ratio, FileError> {
    match n.0 >> 126 {
        0 => Ok(Ratio {
            numer: BigInt::from(interpret_small_int(n.0)),
            denom: BigInt::one(),
        }),
        1 => {
            let (numer, denom) = interpret_small_ratio(n.0);
            Ok(Ratio {
                numer: BigInt::from(numer),
                denom: BigInt::from(denom),
            })
        },
        2 => {
            let numer: BigInt = unintern_bytes(n.0, intermediate_dir)?;
            Ok(Ratio { numer, denom: BigInt::one() })
        },
        3 => unintern_bytes(n.0, intermediate_dir),
        _ => unreachable!(),
    }
}

pub fn intern_ratio(n: &Ratio, is_integer: bool, intermediate_dir: &str) -> Result<InternedNumber, FileError> {
    match (i128::try_from(&n.numer), i128::try_from(&n.denom)) {
        // SmallInt
        (Ok(n @ -21267647932558653966460912964485513216..=21267647932558653966460912964485513215), Ok(1)) => Ok(InternedNumber(
            ((is_integer as u128) << 125) | (n as u128) & SMALL_INT_PAYLOAD_MASK,
        )),
        // SmallRatio
        (Ok(numer @ -4611686018427387904..=4611686018427387903), Ok(denom @ 0..=4611686018427387903)) => Ok(InternedNumber(
            ((is_integer as u128) << 125) | SMALL_RATIO | (((numer as u128) & SMALL_RATIO_NUMER_PAYLOAD_MASK) << 62) | denom as u128,
        )),
        // BigInt
        (_, Ok(1)) => intern_big_int(&n.numer, is_integer, intermediate_dir),
        // BigRatio
        _ => {
            let id = intern_bytes(&n.encode(), intermediate_dir)? & 0xffff_ffff_ffff_ffff_ffff_ffff;
            Ok(InternedNumber(((is_integer as u128) << 125) | BIG_RATIO | id))
        },
    }
}

pub fn intern_big_int(n: &BigInt, is_integer: bool, intermediate_dir: &str) -> Result<InternedNumber, FileError> {
    match i128::try_from(n) {
        Ok(n @ -21267647932558653966460912964485513216..=21267647932558653966460912964485513215) => Ok(InternedNumber(
            ((is_integer as u128) << 125) | (n as u128) & SMALL_INT_PAYLOAD_MASK,
        )),
        _ => {
            let id = intern_bytes(&n.encode(), intermediate_dir)? & 0xffff_ffff_ffff_ffff_ffff_ffff;
            Ok(InternedNumber(((is_integer as u128) << 125) | BIG_INT | id))
        },
    }
}

/// Lexer must guarantee that it's parse-able.
pub fn intern_number_raw(
    base: Base,
    integer: &[u8],

    // `frac` is always decimal
    frac: &[u8],
    exp: i64,

    // of the original literal
    is_integer: bool,

    intermediate_dir: &str,
) -> Result<InternedNumber, FileError> {
    match (base, frac.len(), exp) {
        (Base::Hexadecimal, 0, 0) => match u128::from_str_radix(&String::from_utf8_lossy(integer), 16) {
            Ok(n @ 0..=21267647932558653966460912964485513215) => Ok(InternedNumber(
                ((is_integer as u128) << 125) | n,
            )),
            Ok(n) => intern_big_int(&BigInt::from(n), is_integer, intermediate_dir),
            Err(_) => {
                let n = BigInt::parse_positive_hex(integer).unwrap();
                intern_big_int(&n, is_integer, intermediate_dir)
            },
        },
        (Base::Hexadecimal, _, _) => unreachable!(),
        (Base::Decimal, 0, 0) => match String::from_utf8_lossy(integer).parse::<u128>() {
            Ok(n @ 0..=21267647932558653966460912964485513215) => Ok(InternedNumber(
                ((is_integer as u128) << 125) | n,
            )),
            Ok(n) => intern_big_int(&BigInt::from(n), is_integer, intermediate_dir),
            Err(_) => {
                let n = BigInt::parse_positive_decimal(integer).unwrap();
                intern_big_int(&n, is_integer, intermediate_dir)
            },
        },
        (Base::Decimal, _, _) => {
            let mut integer = BigInt::parse_positive_decimal(integer).unwrap();
            let mut frac_numer = match frac.len() {
                0 => BigInt::zero(),
                _ => BigInt::parse_positive_decimal(frac).unwrap(),
            };
            let mut frac_denom = powi_ubi(&[10], frac.len() as u32);

            let r = gcd_ubi(&frac_numer.nums, &frac_denom);
            frac_numer.nums = div_ubi(&frac_numer.nums, &r);
            frac_denom = div_ubi(&frac_denom, &r);

            let (mut numer, mut denom) = match exp {
                ..0 => match u32::try_from(-exp) {
                    Ok(exp) => {
                        let power = powi_ubi(&[10], exp);
                        let numer = add_ubi(&mul_ubi(&integer.nums, &frac_denom), &frac_numer.nums);
                        let denom = mul_ubi(&frac_denom, &power);
                        (numer, denom)
                    },
                    Err(_) => todo!(),
                },
                _ => match u32::try_from(exp) {
                    Ok(exp) => {
                        let power = powi_ubi(&[10], exp);
                        integer.nums = mul_ubi(&integer.nums, &power);
                        frac_numer.nums = mul_ubi(&frac_numer.nums, &power);
                        let numer = add_ubi(&mul_ubi(&integer.nums, &frac_denom), &frac_numer.nums);
                        let denom = frac_denom;
                        (numer, denom)
                    },
                    Err(_) => todo!(),
                },
            };

            let r = gcd_ubi(&numer, &denom);
            numer = div_ubi(&numer, &r);
            denom = div_ubi(&denom, &r);

            intern_ratio(
                &Ratio {
                    numer: BigInt { is_neg: false, nums: numer },
                    denom: BigInt { is_neg: false, nums: denom },
                },
                is_integer,
                intermediate_dir,
            )
        },
        (Base::Octal, 0, 0) => match u128::from_str_radix(&String::from_utf8_lossy(integer), 8) {
            Ok(n @ 0..=21267647932558653966460912964485513215) => Ok(InternedNumber(
                ((is_integer as u128) << 125) | n,
            )),
            Ok(n) => intern_big_int(&BigInt::from(n), is_integer, intermediate_dir),
            Err(_) => todo!(),
        },
        (Base::Octal, _, _) => unreachable!(),
        (Base::Binary, 0, 0) => match u128::from_str_radix(&String::from_utf8_lossy(integer), 2) {
            Ok(n @ 0..=21267647932558653966460912964485513215) => Ok(InternedNumber(
                ((is_integer as u128) << 125) | n,
            )),
            Ok(n) => intern_big_int(&BigInt::from(n), is_integer, intermediate_dir),
            Err(_) => todo!(),
        },
        (Base::Binary, _, _) => unreachable!(),
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    }

    else {
        gcd(b, a % b)
    }
}

fn interpret_small_int(n: u128) -> i128 {
    let mut payload = n & SMALL_INT_PAYLOAD_MASK;

    if payload >> 124 > 0 {
        payload |= 7 << 125;
    }

    payload as i128
}

fn interpret_small_ratio(n: u128) -> (i64, u64) {
    let denom = (n & 0x3fff_ffff_ffff_ffff) as u64;
    let mut numer = ((n >> 62) & 0x7fff_ffff_ffff_ffff) as u64;

    if numer >> 62 > 0 {
        numer |= 0x8000_0000_0000_0000;
    }

    let numer = numer as i64;
    (numer, denom)
}

fn intern_bytes(bytes: &[u8], intermediate_dir: &str) -> Result<u128, FileError> {
    let id = hash(bytes) & 0xffff_ffff_ffff_ffff_ffff_ffff;
    let id_str = format!("{id:024x}");
    let path = join3(intermediate_dir, "num", &id_str)?;
    write_bytes(
        &path,
        bytes,
        WriteMode::CreateOrTruncate,
    )?;
    Ok(id)
}

fn unintern_bytes<T: Endec>(id: u128, intermediate_dir: &str) -> Result<T, FileError> {
    let id = id & 0xffff_ffff_ffff_ffff_ffff_ffff;
    let id_str = format!("{id:024x}");
    let path = join3(intermediate_dir, "num", &id_str)?;
    let bytes = read_bytes(&path)?;
    T::decode(&bytes).map_err(|_| FileError {
        kind: FileErrorKind::CannotDecodeFile,
        given_path: Some(path.to_string()),
    })
}

// It doesn't need intermediate_dir because `u8` fits in the small_int range!
macro_rules! try_from_interned_number {
    ($ty:ty, $min:literal, $max:literal) => {
        impl TryFrom<InternedNumber> for $ty {
            type Error = ();

            fn try_from(n: InternedNumber) -> Result<$ty, ()> {
                match n.0 >> 126 {
                    0 => match interpret_small_int(n.0) {
                        n @ $min..=$max => Ok(n as $ty),
                        _ => Err(()),
                    },
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_interned_number!(u8, 0, 255);
try_from_interned_number!(u16, 0, 65535);
try_from_interned_number!(u32, 0, 4294967295);
try_from_interned_number!(u64, 0, 18446744073709551615);

try_from_interned_number!(i8, -128, 127);
try_from_interned_number!(i16, -32768, 32767);
try_from_interned_number!(i32, -2147483648, 2147483647);
try_from_interned_number!(i64, -9223372036854775808, 9223372036854775807);
