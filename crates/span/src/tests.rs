use crate::{PolySpanKind, Span, SpanDeriveKind};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<PolySpanKind>() <= 32, "{}", size_of::<PolySpanKind>());
    assert!(size_of::<SpanDeriveKind>() <= 32, "{}", size_of::<SpanDeriveKind>());
    // 32 bytes when mono_id is either u128 or u64
    assert!(size_of::<Span>() <= 32, "{}", size_of::<Span>());
}
