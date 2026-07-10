use super::optimize_local;
use sodigy_bytecode::{Bytecode, Memory, Offset};

#[test]
fn t1() {
    // TODO: impl bytecode-parser
    let unoptimized: Vec<Bytecode> = vec![
        Bytecode::InitTuple {
            elements: 2,
            dst: Memory::SSA(2),
            debug_info: None,
        },
        Bytecode::Move {
            src: Memory::SSA(0),
            dst: Memory::Heap {
                ptr: Box::new(Memory::SSA(2)),
                offset: Offset::Static(0),
            },
        },
        Bytecode::Move {
            src: Memory::SSA(1),
            dst: Memory::Heap {
                ptr: Box::new(Memory::SSA(2)),
                offset: Offset::Static(1),
            },
        },
        Bytecode::Move {
            src: Memory::Heap {
                ptr: Box::new(Memory::SSA(2)),
                offset: Offset::Static(0),
            },
            dst: Memory::SSA(7),
        },
        Bytecode::Return(7),
    ];
    let mut optimized = unoptimized.clone();

    for _ in 0..5 {
        optimize_local(&mut optimized);
    }

    let expected: Vec<Bytecode> = vec![
        Bytecode::Return(0),
    ];

    let unoptimized = unoptimized.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("\n");
    let unoptimized = unoptimized.trim().to_string();
    let optimized = optimized.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("\n");
    let optimized = optimized.trim().to_string();
    let expected = expected.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("\n");
    let expected = expected.trim().to_string();

    if optimized != expected {
        panic!("--- unoptimized ---\n{unoptimized}\n--- optimized ---\n{optimized}\n--- expected ---\n{expected}");
    }
}
