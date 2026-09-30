use crate::{Heap, Stack};
use sodigy_bytecode::{
    BasicBlock,
    CodeSection,
    GlobalLabel,
    LocalLabel,
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
    Span,
    render_spans,
};
use std::collections::{HashMap, HashSet};
use std::io::{Write, self};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Context {
    EnterEntry,
    EnterCodeSection,
    EnterBasicBlock,
    Bytecode(usize),
    Terminator,
}

#[derive(Clone, Copy, Debug)]
pub enum SkipUntil {
    // skips if the call_stack depth is deeper than this
    Bytecode { stack: usize },
    BasicBlock { stack: usize },
    CodeSection { stack: usize },

    Entry,
    Forever,
}

pub struct Session {
    pub call_stack: Vec<GlobalLabel>,
    pub func_spans: HashMap<GlobalLabel, Span>,

    breakpoints: HashSet<(GlobalLabel, LocalLabel)>,
    span_option: RenderSpanOption,
    span_session: RenderSpanSession,
    dump_history: Vec<Buffer>,
    skip_until: Option<SkipUntil>,
    auto_run: bool,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Self {
        Session {
            call_stack: vec![],
            func_spans: HashMap::new(),
            breakpoints: HashSet::new(),
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
            auto_run: false,
        }
    }

    pub fn dump(
        &mut self,
        stack: &Stack,
        heap: &Heap,
        code: Option<&CodeSection>,
        basic_block: Option<&BasicBlock>,
        context: Context,
    ) {
        let mut reached_breakpoint = false;
        let mut in_breakpoint = false;

        match (self.call_stack.last(), basic_block) {
            (
                Some(global_label),
                Some(BasicBlock { label: local_label, .. }),
            ) if self.breakpoints.contains(&(*global_label, *local_label)) => {
                in_breakpoint = true;

                if context == Context::EnterBasicBlock {
                    self.skip_until = None;
                    self.auto_run = false;
                    reached_breakpoint = true;
                }
            },
            _ => {},
        };

        match (self.skip_until, context) {
            (Some(SkipUntil::Bytecode { stack }), _) if stack >= self.call_stack.len() => {
                self.skip_until = None;
            },
            (Some(SkipUntil::Bytecode { .. }), _) => return,
            (Some(SkipUntil::BasicBlock { stack }), Context::EnterBasicBlock | Context::EnterCodeSection | Context::EnterEntry) if stack >= self.call_stack.len() => {
                self.skip_until = None;
            },
            (Some(SkipUntil::BasicBlock { .. }), _) => return,
            (Some(SkipUntil::CodeSection { stack }), Context::EnterCodeSection | Context::EnterEntry) if stack >= self.call_stack.len() => {
                self.skip_until = None;
            },
            (Some(SkipUntil::CodeSection { .. }), _) => return,
            (Some(SkipUntil::Entry), Context::EnterEntry) => {
                self.skip_until = None;
            },
            (Some(SkipUntil::Entry), _) => return,
            (Some(SkipUntil::Forever), _) => return,
            _ => {},
        }

        // There's nothing to show!
        if let Context::EnterEntry = context {
            return;
        }

        let mut spans: Vec<RenderableSpan> = vec![];

        if let Some(label) = self.call_stack.last() {
            spans.push(RenderableSpan {
                span: self.func_spans.get(label).unwrap().clone(),
                auxiliary: true,
                note: Some(String::from("code")),
            });
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

        let mut buffer_top = vec![];
        let mut buffer_left = vec![];
        let mut buffer_right = vec![];
        let mut buffer_bottom = vec![];

        if reached_breakpoint {
            buffer_top.push(format!("---- Breakpoint ----\n"));
        } else {
            buffer_top.push(format!("---- {context:?} ----\n"));

            if let Context::EnterCodeSection = context && let Some(label) = self.call_stack.last() {
                buffer_top.push(format!("label: {}\n", label.hex(20)));
            }
        }

        if !spans.is_empty() {
            let s = render_spans(
                &spans,
                &self.span_option,
                &mut self.span_session,
            );
            buffer_left.push(s);
        }

        if let Some(basic_block) = basic_block {
            buffer_bottom.push(String::from("--- SSA ---\n"));
            let mut used_ssa_indexes: Vec<SSA> = vec![];
            used_ssa_indexes.extend(basic_block.terminator.used_ssa_indexes());

            for bytecode in basic_block.code.iter() {
                used_ssa_indexes.extend(bytecode.used_ssa_indexes());
            }

            used_ssa_indexes.sort();
            used_ssa_indexes.dedup();

            for ssa in used_ssa_indexes.iter() {
                if let Some(value) = stack.ssa.get(ssa) {
                    buffer_bottom.push(format!("{ssa}: {}\n", debug_stack(*value, stack, heap)));
                } else {
                    buffer_bottom.push(format!("{ssa}: N/A\n"));
                }
            }
        }

        buffer_bottom.push(String::from("--- call stack ---\n"));

        for (i, call) in self.call_stack.iter().enumerate() {
            // TODO: calc file_name, row, col
            buffer_bottom.push(format!("{i}. {call:?} // TODO: calc file_name/row/col\n"));
        }

        if let Some(code) = code {
            let highlight = match (basic_block, context) {
                (Some(BasicBlock { label, .. }), Context::EnterBasicBlock) => Some((*label, Highlight::Label)),
                (Some(BasicBlock { label, .. }), Context::Bytecode(i)) => Some((*label, Highlight::Bytecode(i))),
                (Some(BasicBlock { label, .. }), Context::Terminator) => Some((*label, Highlight::Terminator)),
                _ => None,
            };
            let mut object_file_dump = code.dump(true, highlight, Some(8), false);

            if object_file_dump.lines().count() > 20 {
                object_file_dump = object_file_dump.lines().take(20).map(
                    |line| line.to_string()
                ).collect::<Vec<_>>().join("\n");
            }

            buffer_right.push(object_file_dump);
        }

        self.dump_history.push(Buffer {
            top: buffer_top.concat(),
            left: buffer_left.concat(),
            right: buffer_right.concat(),
            bottom: buffer_bottom.concat(),
        });

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
                println!("{}", self.dump_history[cursor].render(72, " |", 72));
            }

            let commands = if let Overlay::Full(_) = &overlay {
                vec![
                    Some("q: close"),
                ]
            } else if watching_history {
                vec![
                    if cursor > 0 { Some("n: see previous dump") } else { None },
                    Some("m: go to current dump"),
                ]
            } else {
                vec![
                    Some("a: next breakpoint (show trace)"),
                    Some("s: next breakpoint (hide trace)"),
                    if self.call_stack.is_empty() || basic_block.is_none() {
                        None
                    } else if in_breakpoint {
                        Some("d: remove breakpoint")
                    } else {
                        Some("d: set breakpoint")
                    },
                    Some("z: next bytecode (or press any key)"),
                    Some("x: next bytecode, but don't jump into another function"),
                    Some("c: next basic block"),
                    Some("v: next code section"),
                    Some("b: next entry"),
                    Some("hN: inspect heap, at address N"),
                    if cursor > 0 { Some("n: see previous dump") } else { None },
                ]
            };

            println!("");

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

            if self.auto_run {
                command = String::from("a");
            }

            else {
                std::io::stdin().read_line(&mut command).unwrap();
            }

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
                    "n" => {
                        cursor -= 1;
                    },
                    "m" => {
                        cursor = self.dump_history.len() - 1;
                        watching_history = false;
                    },
                    _ => {},
                }

                continue;
            } else {
                match command.trim() {
                    "a" => {
                        self.auto_run = true;
                    },
                    "s" => {
                        self.skip_until = Some(SkipUntil::Forever);
                    },
                    "d" => {
                        if let (Some(global_label), Some(BasicBlock { label: local_label, .. })) = (self.call_stack.last(), basic_block) {
                            if in_breakpoint {
                                self.breakpoints.remove(&(*global_label, *local_label));
                                in_breakpoint = false;
                            } else {
                                self.breakpoints.insert((*global_label, *local_label));
                                in_breakpoint = true;
                            }

                            continue;
                        }
                    },
                    "x" => {
                        self.skip_until = Some(SkipUntil::Bytecode { stack: self.call_stack.len() });
                    },
                    "c" => {
                        self.skip_until = Some(SkipUntil::BasicBlock { stack: self.call_stack.len() });
                    },
                    "v" => {
                        self.skip_until = Some(SkipUntil::CodeSection { stack: self.call_stack.len() });
                    },
                    "b" => {
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
                    "n" if cursor > 0 => {
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

#[derive(Clone, Debug)]
struct Buffer {
    top: String,
    left: String,
    right: String,
    bottom: String,
}

impl Buffer {
    pub fn render(&self, left: usize, delim: &str, right: usize) -> String {
        fn set_len(s: &str, l: usize) -> String {
            let mut buffer = vec![];
            let s = s.as_bytes();
            let mut i = 0;
            let mut line_len = 0;
            let mut wait_until_m = false;

            loop {
                match (s.get(i), s.get(i + 1)) {
                    // ANSI coloring
                    (Some(27), Some(b'[')) => {
                        buffer.push(27);
                        buffer.push(b'[');
                        i += 2;

                        loop {
                            match s.get(i) {
                                Some(b'm') => {
                                    buffer.push(b'm');
                                    i += 1;
                                    break;
                                },
                                Some(b) => {
                                    buffer.push(*b);
                                    i += 1;
                                },
                                None => {
                                    break;
                                },
                            }
                        }
                    },
                    (Some(b), _) => {
                        buffer.push(*b);
                        i += 1;
                        line_len += 1;
                    },
                    (None, _) => {
                        while line_len < l {
                            buffer.push(b' ');
                            line_len += 1;
                        }
                    },
                }

                if line_len == l {
                    break;
                }
            }

            String::from_utf8(buffer).unwrap()
        }

        let mut lines = vec![];

        for line in self.top.lines() {
            lines.push(line.to_string());
        }

        let left_lines: Vec<_> = self.left.lines().collect();
        let right_lines: Vec<_> = self.right.lines().collect();

        for i in 0..(left_lines.len().max(right_lines.len())) {
            let left_line = left_lines.get(i).unwrap_or(&"");
            let right_line = right_lines.get(i).unwrap_or(&"");
            lines.push(format!("{}{delim}{}", set_len(left_line, left), set_len(right_line, right)));
        }

        lines.push(String::new());

        for line in self.bottom.lines() {
            lines.push(line.to_string());
        }

        lines.join("\n")
    }
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
