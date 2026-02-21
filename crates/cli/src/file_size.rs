// I can make it 10 times simpler if I use Regex. But,
// 1. I don't want to add any dependencies to ragit-cli.
// 2. It's more fun to write a parser from scratch.

use crate::error::{ErrorKind, RawError};
use crate::span::Span;

#[derive(PartialEq)]
enum ParseState {
    Integer,
    Fraction,
    UnitOrB,
    IOrB,
    B,
    Done,
}

// "4 KiB" -> 4096
// "12 MB" -> 12000000
// "9999" -> 9999
pub fn parse_file_size(s: &str, span: Span) -> Result<u64, RawError> {
    let mut state = ParseState::Integer;
    let mut integer_part = vec![];
    let mut fraction_part = vec!['1'];
    let mut unit = None;
    let mut bin_unit = false;

    for c in s.chars() {
        match state {
            ParseState::Integer => match c {
                '0'..='9' => {
                    integer_part.push(c);
                },
                ' ' => {
                    state = ParseState::UnitOrB;
                },
                '.' => {
                    state = ParseState::Fraction;
                },
                'k' | 'K'
                | 'm' | 'M'
                | 'g' | 'G'
                | 't' | 'T'
                | 'p' | 'P' => {
                    unit = Some(c.to_ascii_lowercase());
                    state = ParseState::IOrB;
                },
                'b' | 'B' => {
                    state = ParseState::Done;
                },
                _ => {
                    return Err(RawError {
                        span,
                        kind: ErrorKind::ParseFileSizeError,
                    });
                },
            },
            ParseState::Fraction => match c {
                '0'..='9' => {
                    fraction_part.push(c);
                },
                ' ' => {
                    state = ParseState::UnitOrB;
                },
                'k' | 'K'
                | 'm' | 'M'
                | 'g' | 'G'
                | 't' | 'T'
                | 'p' | 'P' => {
                    unit = Some(c.to_ascii_lowercase());
                    state = ParseState::IOrB;
                },
                'b' | 'B' => {
                    state = ParseState::Done;
                },
                _ => {
                    return Err(RawError {
                        span,
                        kind: ErrorKind::ParseFileSizeError,
                    });
                },
            },
            ParseState::UnitOrB => match c {
                'k' | 'K'
                | 'm' | 'M'
                | 'g' | 'G'
                | 't' | 'T'
                | 'p' | 'P' => {
                    unit = Some(c.to_ascii_lowercase());
                    state = ParseState::IOrB;
                },
                'b' | 'B' => {
                    state = ParseState::Done;
                },
                _ => {
                    return Err(RawError {
                        span,
                        kind: ErrorKind::ParseFileSizeError,
                    });
                },
            },
            ParseState::IOrB => match c {
                'i' | 'I' => {
                    bin_unit = true;
                    state = ParseState::B;
                },
                'b' | 'B' => {
                    state = ParseState::Done;
                },
                _ => {
                    return Err(RawError {
                        span,
                        kind: ErrorKind::ParseFileSizeError,
                    });
                },
            },
            ParseState::B => match c {
                'b' | 'B' => {
                    state = ParseState::Done;
                },
                _ => {
                    return Err(RawError {
                        span,
                        kind: ErrorKind::ParseFileSizeError,
                    });
                },
            },
            ParseState::Done => {
                return Err(RawError {
                    span,
                    kind: ErrorKind::ParseFileSizeError,
                });
            },
        }
    }

    let integer_s = integer_part.iter().collect::<String>();
    let integer = match integer_s.parse::<u64>() {
        Ok(n) => n,
        Err(e) => {
            return Err(RawError {
                span,
                kind: ErrorKind::ParseIntError(e),
            });
        },
    };

    // 0 <= fraction < 10^15
    let fraction = if fraction_part.len() == 1 {
        0
    } else {
        while fraction_part.len() < 16 {
            fraction_part.push('0');
        }

        while fraction_part.len() > 16 {
            fraction_part.pop().unwrap();
        }

        let fraction_s = fraction_part.iter().collect::<String>();
        let fraction = match fraction_s.parse::<u64>() {
            Ok(n) => n,
            Err(_) => {
                return Err(RawError {
                    span,
                    kind: ErrorKind::ParseFileSizeError,
                });
            },
        };

        fraction - 1_000_000_000_000_000
    };

    let mul: u64 = match (unit, bin_unit) {
        (None, _) => 1,
        (Some('k'), true) => 1 << 10,
        (Some('k'), false) => 1_000,
        (Some('m'), true) => 1 << 20,
        (Some('m'), false) => 1_000_000,
        (Some('g'), true) => 1 << 30,
        (Some('g'), false) => 1_000_000_000,
        (Some('t'), true) => 1 << 40,
        (Some('t'), false) => 1_000_000_000_000,
        (Some('p'), true) => 1 << 50,
        (Some('p'), false) => 1_000_000_000_000_000,
        _ => unreachable!(),
    };

    if integer > 0 && integer.ilog2() + mul.ilog2() > 61 {
        return Err(RawError {
            span,
            kind: ErrorKind::ParseFileSizeError,
        });
    }

    let mut result = integer * mul;

    if fraction != 0 && mul > 1 {
        result += (mul as u128 * fraction as u128 / 1_000_000_000_000_000) as u64;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::parse_file_size;
    use crate::span::Span;

    #[test]
    fn parse_file_size_test() {
        let sample = vec![
            ("", None),
            ("k", None),
            ("b", None),
            ("0", Some(0)),
            ("0B", Some(0)),
            ("0KB", Some(0)),
            ("100", Some(100)),
            ("2Kb", Some(2000)),
            ("2Kib", Some(2048)),
            ("2Kiib", None),
            ("2Kbb", None),
            ("2 Kb", Some(2000)),
            ("2 Kib", Some(2048)),
            ("2k", Some(2000)),
            ("2 k", Some(2000)),
            ("2ki", Some(2048)),
            ("2i", None),
            ("2b", Some(2)),
            ("1.5Kb", Some(1500)),
            ("1.5Kib", Some(1536)),
            ("1.5 Kb", Some(1500)),
            ("1.5 Kib", Some(1536)),
            ("1.5", Some(1)),
            ("1.5 ", Some(1)),
            ("1.5B", Some(1)),
            ("1.5 B", Some(1)),
            ("0.1", Some(0)),
            ("0.5", Some(0)),
            ("0.5B", Some(0)),
            ("0.0012MB", Some(1200)),
            ("0.01MB", Some(10000)),
            ("0.01MiB", Some(10485)),
            ("1.01MB", Some(1010000)),
            ("1.1MB", Some(1100000)),
            ("1.2MB", Some(1200000)),
            ("1.3MB", Some(1300000)),
            ("1.4MB", Some(1400000)),
            ("1.5MB", Some(1500000)),
            ("1.6MB", Some(1600000)),
            ("1.7MB", Some(1700000)),
            ("1.8MB", Some(1800000)),
            ("1.9MB", Some(1900000)),
            ("1.01MiB", Some(1059061)),
            ("1.1MiB", Some(1153433)),
            ("1.2MiB", Some(1258291)),
            ("1.3MiB", Some(1363148)),
            ("1.4MiB", Some(1468006)),
            ("1.5MiB", Some(1572864)),
            ("1.6MiB", Some(1677721)),
            ("1.7MiB", Some(1782579)),
            ("1.8MiB", Some(1887436)),
            ("1.9MiB", Some(1992294)),
            ("123.45678GB", Some(123456780000)),
        ];

        for (input, answer) in sample.into_iter() {
            let result = parse_file_size(input, Span::None);

            if let Some(answer) = answer {
                // I don't want to `#[derive(Debug)]` for Error.
                // I mean, I'll derive it someday, but it's a complicated issue
                // and beyond the scope of this test.
                assert_eq!(result.map_err(|_| 0).unwrap(), answer);
            }

            else {
                assert!(result.is_err());
            }
        }
    }
}
