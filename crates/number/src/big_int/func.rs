pub fn ilog2_ubi(ns: &[u32]) -> u32 {
    match ns.last() {
        Some(n) => n.ilog2() + (ns.len() - 1) as u32 * 32,
        None => unreachable!(),
    }
}
