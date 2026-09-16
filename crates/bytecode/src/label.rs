use sodigy_span::SpanHash;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalLabel(u32);

impl LocalLabel {
    pub fn new(n: u32) -> Self {
        LocalLabel(n)
    }

    pub fn start() -> Self {
        LocalLabel(0)
    }

    pub fn index(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GlobalLabel(SpanHash);

impl GlobalLabel {
    pub fn new(s: SpanHash) -> Self {
        GlobalLabel(s)
    }

    pub fn hex(&self, l: usize) -> String {
        self.0.hex(l)
    }

    pub fn span(&self) -> SpanHash {
        self.0
    }
}
