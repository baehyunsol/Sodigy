use crate::{Heap, Stack};
use sodigy_bytecode::Bytecode;
use sodigy_number::bi_to_string;
use sodigy_span::{RenderableSpan, RenderSpanOption, RenderSpanSession, render_spans};
use std::collections::HashSet;

pub fn debug(
    stack: &Stack,
    heap: &Heap,
    bytecodes: &[Bytecode],
    cursor: usize,
    render_span_session: &mut RenderSpanSession,
) {
    let mut stack_preview: Vec<&u32> = stack.ssa.keys().collect();
    let mut total_stack_size = stack_preview.len();
    let mut too_many_ssas = None;
    stack_preview.sort();

    if stack_preview.len() > 8 {
        let mut interesting_stack = HashSet::new();

        for c in (cursor.max(2) - 2)..(cursor + 3).min(bytecodes.len()) {
            for s in bytecodes[c].used_ssa_indexes() {
                interesting_stack.insert(s);
            }
        }

        stack_preview = stack_preview.into_iter().filter(
            |s| interesting_stack.contains(s)
        ).collect();
    }

    if stack_preview.len() > 8 {
        too_many_ssas = Some(total_stack_size - 8);
        stack_preview = stack_preview[..8].to_vec();
    }

    println!("\n\n{}\n", "-".repeat(64));

    if let Some(debug_info) = bytecodes[cursor].debug_info() {
        let s = render_spans(
            &[RenderableSpan {
                span: *debug_info.clone(),
                auxiliary: true,
                note: None,
            }],
            &RenderSpanOption {
                max_height: 10,
                max_width: 88,
                context: 5,
                render_source: true,
                color: None,
                group_delim: None,
            },
            render_span_session,
        );

        if s.trim() != "" {
            println!("\n{s}\n");
        }
    }

    println!("_ret: {}", debug_stack(stack.r#return, stack, heap));

    for s in stack_preview {
        if let Some(ss) = stack.ssa.get(s) {
            println!("_{s}: {}", debug_stack(*ss, stack, heap));
        }
    }

    if let Some(n) = too_many_ssas {
        println!("... (truncated {n} ssas)");
    }

    println!();

    for c in (cursor.max(4) - 4)..(cursor + 5).min(bytecodes.len()) {
        if c == cursor {
            println!("{} |", if cursor + 2 > 1000 { "       " } else { "     " });
        }

        println!(
            "{}{} | {}{}",
            if c == cursor { "->" } else { "  " },
            if cursor + 2 > 1000 { format!("{c:>5}") } else { format!("{c:>3}") },
            if let Bytecode::Label(_) = &bytecodes[c] { "" } else { "    " },
            &bytecodes[c],
        );

        if c == cursor {
            println!("{} |", if cursor + 2 > 1000 { "       " } else { "     " });
        }
    }

    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn debug_stack(value: u32, stack: &Stack, heap: &Heap) -> String {
    let int = match try_inspect_int(&heap.data, value as usize) {
        Some((is_neg, ns)) => bi_to_string(is_neg, ns),
        None => String::from("????"),
    };
    let string = match try_inspect_list(&heap.data, value as usize) {
        Some(s) => {
            let (ss, truncated) = if s.len() > 12 { (&s[..12], true) } else { (s, false) };
            format!(
                "{:?}{}",
                ss.iter().map(
                    |ch| char::from_u32(*ch).unwrap_or('�')
                ).collect::<String>(),
                if truncated {
                    format!("...(truncated {} chars)", s.len() - 12)
                } else {
                    String::new()
                },
            )
        },
        None => String::from("????"),
    };
    let ref_count = if value > 0 {
        heap.data[value as usize - 1].to_string()
    } else {
        String::from("????")
    };

    format!("scalar={value}, int={int}, string={string}, ref_count={ref_count}")
}

fn try_inspect_int(heap: &[u32], ptr: usize) -> Option<(bool, &[u32])> {
    if ptr >= heap.len() {
        return None;
    }

    let metadata = heap[ptr];
    let is_neg = metadata > 0x7fff_ffff;
    let length = metadata & 0x7fff_ffff;

    if length != 0 && length < 32 {
        Some((is_neg, &heap[(ptr + 1)..(ptr + 1 + length as usize)]))
    }

    else {
        None
    }
}

fn try_inspect_list(heap: &[u32], ptr: usize) -> Option<&[u32]> {
    if ptr + 2 >= heap.len() {
        return None;
    }

    let slice_ptr = heap[ptr] as usize;
    let start = heap[ptr + 1] as usize;
    let length = heap[ptr + 2] as usize;

    if slice_ptr + start + length + 1 >= heap.len() {
        None
    }

    else {
        Some(&heap[(slice_ptr + start + 1)..(slice_ptr + start + length + 1)])
    }
}
