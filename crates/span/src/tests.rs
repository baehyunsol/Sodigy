use crate::{PolySpanKind, Span, SpanDeriveKind};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<PolySpanKind>() < 32, "{}", size_of::<PolySpanKind>());
    assert!(size_of::<SpanDeriveKind>() < 32, "{}", size_of::<SpanDeriveKind>());
    assert!(size_of::<Span>() < 48, "{}", size_of::<Span>());
}
