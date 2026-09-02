pub fn dump_hex(n: u128, l: usize) -> String {
    // damn...
    match l {
        1 => format!("{:01x}", n & 0xf),
        2 => format!("{:02x}", n & 0xff),
        3 => format!("{:03x}", n & 0xfff),
        4 => format!("{:04x}", n & 0xffff),
        5 => format!("{:05x}", n & 0xf_ffff),
        6 => format!("{:06x}", n & 0xff_ffff),
        7 => format!("{:07x}", n & 0xfff_ffff),
        8 => format!("{:08x}", n & 0xffff_ffff),
        9 => format!("{:09x}", n & 0xf_ffff_ffff),
        10 => format!("{:010x}", n & 0xff_ffff_ffff),
        11 => format!("{:011x}", n & 0xfff_ffff_ffff),
        12 => format!("{:012x}", n & 0xffff_ffff_ffff),

        // I'm too lazy to type the rest...
        _ => panic!(),
    }
}

// It's just a toy hash function.
// I'll implement a better one when the project becomes serious.
pub fn hash(s: &[u8]) -> u128 {
    let mut r = 0;

    for (i, b) in s.iter().enumerate() {
        let c = (((r >> 24) & 0x00ff_ffff) << 24) | ((i & 0xfff) << 12) as u128 | *b as u128;
        let cc = c * c + c + 1;
        r += cc;
        r &= 0xffff_ffff_ffff_ffff_ffff_ffff;
    }

    r
}
