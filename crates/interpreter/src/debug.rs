use crate::{Heap, Stack};
use sodigy_bytecode::{
    BasicBlock,
    Bytecode,
    CodeSection,
    Highlight,
    SSA,
};
use sodigy_number::bi_to_string;
use sodigy_span::{
    Color,
    ColorOption,
    RenderableSpan,
    RenderSpanOption,
    RenderSpanSession,
    render_spans,
};
use std::collections::HashSet;
use std::io::{Write, self};

#[derive(Clone, Copy, Debug)]
pub enum Context {
    EnterEntry,
    EnterCodeSection,
    EnterBasicBlock,
    Bytecode(usize),
    Terminator,
}

#[derive(Clone, Copy, Debug)]
pub enum SkipUntil {
    BasicBlock,
    CodeSection,
    Entry,
}

pub struct Session {
    pub code_section: Option<CodeSection>,
    span_option: RenderSpanOption,
    span_session: RenderSpanSession,
    dump_history: Vec<String>,
    skip_until: Option<SkipUntil>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Self {
        Session {
            code_section: None,
            span_option: RenderSpanOption {
                max_height: 20,
                max_width: 96,
                context: 5,
                render_source: true,
                color: Some(ColorOption {
                    primary: Color::Yellow,
                    auxiliary: Color::Yellow,
                    info: Color::Green,
                }),
                group_delim: None,
            },
            span_session: RenderSpanSession::new(intermediate_dir),
            dump_history: vec![],
            skip_until: None,
        }
    }

    pub fn dump(
        &mut self,
        stack: &Stack,
        heap: &Heap,
        basic_block: Option<&BasicBlock>,
        context: Context,
    ) {
        match (self.skip_until, context) {
            (Some(SkipUntil::BasicBlock), Context::EnterBasicBlock | Context::EnterCodeSection | Context::EnterEntry) => {
                self.skip_until = None;
            },
            (Some(SkipUntil::BasicBlock), _) => {
                return;
            },
            (Some(SkipUntil::CodeSection), Context::EnterCodeSection | Context::EnterEntry) => {
                self.skip_until = None;
            },
            (Some(SkipUntil::CodeSection), _) => {
                return;
            },
            (Some(SkipUntil::Entry), Context::EnterEntry) => {
                self.skip_until = None;
            },
            (Some(SkipUntil::Entry), _) => {
                return;
            },
            _ => {},
        }

        // There's nothing to show!
        if let Context::EnterEntry = context {
            return;
        }

        let mut spans: Vec<RenderableSpan> = vec![];

        if let Some(code) = &self.code_section {
            if let Some(span) = &code.span {
                spans.push(RenderableSpan {
                    span: span.clone(),
                    auxiliary: true,
                    note: Some(String::from("code")),
                });
            }
        }

        if let Context::Bytecode(i) = context
            && let Some(basic_block) = basic_block
            && let Some(bytecode) = basic_block.code.get(i)
            && let Some(debug_info) = bytecode.debug_info() {
            spans.push(RenderableSpan {
                span: *debug_info.clone(),
                auxiliary: false,
                note: Some(String::from("bytecode")),
            });
        }

        if let Context::Terminator = context
            && let Some(basic_block) = basic_block
            && let Some(debug_info) = &basic_block.terminator_debug_info {
            spans.push(RenderableSpan {
                span: *debug_info.clone(),
                auxiliary: false,
                note: Some(String::from("terminator")),
            });
        }

        let mut buffer = vec![];
        buffer.push(format!("---- {context:?} ----\n"));

        if !spans.is_empty() {
            let s = render_spans(
                &spans,
                &self.span_option,
                &mut self.span_session,
            );
            buffer.push(format!("{s}\n\n"));
        }

        if let Some(basic_block) = basic_block {
            let mut used_ssa_indexes: Vec<SSA> = vec![];
            used_ssa_indexes.extend(basic_block.terminator.used_ssa_indexes());

            for bytecode in basic_block.code.iter() {
                used_ssa_indexes.extend(bytecode.used_ssa_indexes());
            }

            used_ssa_indexes.sort();
            used_ssa_indexes.dedup();

            for ssa in used_ssa_indexes.iter() {
                if let Some(value) = stack.ssa.get(ssa) {
                    buffer.push(format!("{ssa}: {}\n", debug_stack(*value, stack, heap)));
                } else {
                    buffer.push(format!("{ssa}: N/A\n"));
                }
            }

            buffer.push(String::from("\n"));

            let highlight = match context {
                Context::Bytecode(i) => Highlight::Bytecode(i),
                Context::Terminator => Highlight::Terminator,
                _ => Highlight::None,
            };
            let s = basic_block.dump(true, highlight, false);
            buffer.push(format!("{s}\n\n"));
        }

        self.dump_history.push(buffer.concat());

        while self.dump_history.len() > 100 {
            self.dump_history = self.dump_history[1..].to_vec();
        }

        let mut cursor = self.dump_history.len() - 1;
        let mut watching_history = false;
        let mut overlay = Overlay::None;

        loop {
            print!("\x1b[2J\x1b[H");
            io::stdout().flush().unwrap();

            // TODO: Overlay::Full hasn't been tested
            if let Overlay::Full(s) = &overlay {
                println!("{s}");
            }

            else {
                println!("{}", self.dump_history[cursor]);
            }

            let commands = if let Overlay::Full(_) = &overlay {
                vec![
                    Some("q: close"),
                ]
            } else if watching_history {
                vec![
                    if cursor > 0 { Some("b: see previous dump") } else { None },
                    Some("n: go to current dump"),
                ]
            } else {
                vec![
                    Some("z: next bytecode (or press any key)"),
                    Some("x: next basic block"),
                    Some("c: next code section"),
                    Some("v: next entry"),
                    Some("hN: inspect heap, at address N"),
                    if cursor > 0 { Some("b: see previous dump") } else { None },
                ]
            };

            for command in commands.iter() {
                if let Some(s) = command {
                    println!("{s}");
                }
            }

            if let Overlay::Bottom(s) = &overlay {
                println!("\n{s}\n");
                overlay = Overlay::None;
            }

            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();

            if let Overlay::Full(_) = &overlay {
                match command.trim() {
                    "q" => {
                        overlay = Overlay::None;
                        continue;
                    },
                    _ => {},
                }
            } else if watching_history {
                match command.trim() {
                    "b" => {
                        cursor -= 1;
                    },
                    "n" => {
                        cursor = self.dump_history.len() - 1;
                        watching_history = false;
                    },
                    _ => {},
                }

                continue;
            } else {
                match command.trim() {
                    "x" => {
                        self.skip_until = Some(SkipUntil::BasicBlock);
                    },
                    "c" => {
                        self.skip_until = Some(SkipUntil::CodeSection);
                    },
                    "v" => {
                        self.skip_until = Some(SkipUntil::Entry);
                    },
                    c if c.starts_with("h") => {
                        match c.get(1..) {
                            Some(n) => match n.parse::<u32>() {
                                Ok(n) => {
                                    let s = (0..8).map(
                                        |i| match heap.data.get(n as usize + i) {
                                            Some(value) => value.to_string(),
                                            None => String::from("N/A"),
                                        }
                                    ).collect::<Vec<_>>().join(", ");
                                    overlay = Overlay::Bottom(format!("heap[{n}..] = [{s}, ...]"));
                                },
                                Err(_) => {
                                    overlay = Overlay::Bottom(format!("`{n}` is not a valid 32-bit integer."));
                                },
                            },
                            None => {
                                overlay = Overlay::Bottom(String::from("Cannot parse N."));
                            },
                        }

                        continue;
                    },
                    "b" => {
                        cursor -= 1;
                        watching_history = true;
                        continue;
                    },
                    _ => {},
                }
            }

            break;
        }
    }
}

enum Overlay {
    None,
    Full(String),
    Bottom(String),
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
    let list_meta = {
        let ptr = value as usize;

        if ptr + 2 >= heap.data.len() {
            String::from("????")
        } else {
            let slice_ptr = heap.data[ptr];
            let start = heap.data[ptr + 1];
            let length = heap.data[ptr + 2];
            format!("{{ slice_ptr: {slice_ptr}, start: {start}, length: {length} }}")
        }
    };
    let ref_count = if value > 0 {
        match heap.data.get(value as usize - 1) {
            Some(r) => r.to_string(),
            None => String::from("????"),
        }
    } else {
        String::from("????")
    };

    format!("scalar={value}, int={int}, list_meta={list_meta}, string={string}, ref_count={ref_count}")
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
