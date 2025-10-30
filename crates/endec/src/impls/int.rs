use crate::{DecodeError, Endec};

impl Endec for u8 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(b) => Ok((*b, cursor + 1)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for u16 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        buffer.push((*self >> 8) as u8);
        buffer.push((*self & 0xff) as u8);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match (buffer.get(cursor), buffer.get(cursor + 1)) {
            (Some(x), Some(y)) => Ok((((*x as u16) << 8) | (*y as u16), cursor + 2)),
            _ => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for u32 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        let n = *self;

        if n < (1 << 7) {
            buffer.push(n as u8 | 0x80);
        }

        else if n < (1 << 14) {
            buffer.push((n >> 7) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 21) {
            buffer.push((n >> 14) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 28) {
            buffer.push((n >> 21) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else {
            buffer.push((n >> 28) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }
    }

    fn decode_impl(buffer: &[u8], mut cursor: usize) -> Result<(Self, usize), DecodeError> {
        let mut result = 0;

        loop {
            match buffer.get(cursor) {
                Some(n @ (0..=127)) => {
                    result <<= 7;
                    result += *n as u32;
                    cursor += 1;
                },
                Some(n @ (128..)) => {
                    result <<= 7;
                    result += (*n as u32) & 0x7f;
                    return Ok((result, cursor + 1));
                },
                None => {
                    return Err(DecodeError::UnexpectedEof);
                },
            }
        }
    }
}

impl Endec for u64 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        let n = *self;

        if n < (1 << 28) {
            (n as u32).encode_impl(buffer);
        }

        else if n < (1 << 35) {
            buffer.push((n >> 28) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 42) {
            buffer.push((n >> 35) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 49) {
            buffer.push((n >> 42) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 56) {
            buffer.push((n >> 49) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 63) {
            buffer.push((n >> 56) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else {
            (n as u128).encode_impl(buffer);
        }
    }

    fn decode_impl(buffer: &[u8], mut cursor: usize) -> Result<(Self, usize), DecodeError> {
        let mut result = 0;

        loop {
            match buffer.get(cursor) {
                Some(n @ (0..=127)) => {
                    result <<= 7;
                    result += *n as u64;
                    cursor += 1;
                },
                Some(n @ (128..)) => {
                    result <<= 7;
                    result += (*n as u64) & 0x7f;
                    return Ok((result, cursor + 1));
                },
                None => {
                    return Err(DecodeError::UnexpectedEof);
                },
            }
        }
    }
}

impl Endec for u128 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        let n = *self;

        if n < (1 << 63) {
            (n as u64).encode_impl(buffer);
        }

        else if n < (1 << 70) {
            buffer.push((n >> 63) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 77) {
            buffer.push((n >> 70) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 84) {
            buffer.push((n >> 77) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 91) {
            buffer.push((n >> 84) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 98) {
            buffer.push((n >> 91) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 105) {
            buffer.push((n >> 98) as u8);
            buffer.push(((n >> 91) & 0x7f) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 112) {
            buffer.push((n >> 105) as u8);
            buffer.push(((n >> 98) & 0x7f) as u8);
            buffer.push(((n >> 91) & 0x7f) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 119) {
            buffer.push((n >> 112) as u8);
            buffer.push(((n >> 105) & 0x7f) as u8);
            buffer.push(((n >> 98) & 0x7f) as u8);
            buffer.push(((n >> 91) & 0x7f) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else if n < (1 << 126) {
            buffer.push((n >> 119) as u8);
            buffer.push(((n >> 112) & 0x7f) as u8);
            buffer.push(((n >> 105) & 0x7f) as u8);
            buffer.push(((n >> 98) & 0x7f) as u8);
            buffer.push(((n >> 91) & 0x7f) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }

        else {
            buffer.push((n >> 126) as u8);
            buffer.push(((n >> 119) & 0x7f) as u8);
            buffer.push(((n >> 112) & 0x7f) as u8);
            buffer.push(((n >> 105) & 0x7f) as u8);
            buffer.push(((n >> 98) & 0x7f) as u8);
            buffer.push(((n >> 91) & 0x7f) as u8);
            buffer.push(((n >> 84) & 0x7f) as u8);
            buffer.push(((n >> 77) & 0x7f) as u8);
            buffer.push(((n >> 70) & 0x7f) as u8);
            buffer.push(((n >> 63) & 0x7f) as u8);
            buffer.push(((n >> 56) & 0x7f) as u8);
            buffer.push(((n >> 49) & 0x7f) as u8);
            buffer.push(((n >> 42) & 0x7f) as u8);
            buffer.push(((n >> 35) & 0x7f) as u8);
            buffer.push(((n >> 28) & 0x7f) as u8);
            buffer.push(((n >> 21) & 0x7f) as u8);
            buffer.push(((n >> 14) & 0x7f) as u8);
            buffer.push(((n >> 7) & 0x7f) as u8);
            buffer.push((n & 0x7f) as u8 | 0x80);
        }
    }

    fn decode_impl(buffer: &[u8], mut cursor: usize) -> Result<(Self, usize), DecodeError> {
        let mut result = 0;

        loop {
            match buffer.get(cursor) {
                Some(n @ (0..=127)) => {
                    result <<= 7;
                    result += *n as u128;
                    cursor += 1;
                },
                Some(n @ (128..)) => {
                    result <<= 7;
                    result += (*n as u128) & 0x7f;
                    return Ok((result, cursor + 1));
                },
                None => {
                    return Err(DecodeError::UnexpectedEof);
                },
            }
        }
    }
}

impl Endec for usize {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        (*self as u64).encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (n, cursor) = u64::decode_impl(buffer, cursor)?;
        Ok((n as usize, cursor))
    }
}

impl Endec for i32 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // 0 -> 0
        // -1 -> 1
        // 1 -> 2
        // -2 -> 3
        // 2 -> 4
        // -2147483647 -> 4294967293
        // 2147483647 -> 4294967294
        // -2147483648 -> 4294967295
        let n = if *self < 0 {
            (-(*self as i64) - 1) as u32 * 2 + 1
        } else {
            (*self as u32) * 2
        };

        n.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (n, cursor) = u32::decode_impl(buffer, cursor)?;

        if n % 2 == 0 {
            Ok(((n / 2) as i32, cursor))
        }

        else {
            Ok((-((n / 2) as i32) - 1, cursor))
        }
    }
}

impl Endec for i64 {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // 0 -> 0
        // -1 -> 1
        // 1 -> 2
        // -2 -> 3
        // 2 -> 4
        // -9223372036854775807 -> 18446744073709551613
        // 9223372036854775807 -> 18446744073709551614
        // -9223372036854775808 -> 18446744073709551615
        let n = if *self < 0 {
            (-(*self as i128) - 1) as u64 * 2 + 1
        } else {
            (*self as u64) * 2
        };

        n.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (n, cursor) = u64::decode_impl(buffer, cursor)?;

        if n % 2 == 0 {
            Ok(((n / 2) as i64, cursor))
        }

        else {
            Ok((-((n / 2) as i64) - 1, cursor))
        }
    }
}
