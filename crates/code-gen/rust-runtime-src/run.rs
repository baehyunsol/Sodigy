pub enum CallResult {
    TailCallShort { f: Fn(&mut Heap, u32, u32) -> CallResult, x0: u32, x1: u32 },
    TailCallLong { f: Fn(&mut Heap, u32, u32, Vec<u32>) -> CallResult, x0: u32, x1: u32, xs: Vec<u32> },
    Return(u32),
    Exit(u8),
}

pub fn call(heap: &mut Heap, mut c: CallResult) -> CallResult {
    loop {
        match c {
            CallResult::TailCallShort { f, x0, x1 } => { c = f(x0, x1); },
            CallResult::TailCallLong { f, x0, x1, xs } => { c = f(x0, x1, xs); },
            CallResult::Return(n) => {
                return CallResult::Return(n);
            },
            CallResult::Exit(n) => {
                return CallResult::Exit(n);
            },
        }
    }
}
