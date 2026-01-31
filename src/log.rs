use crate::{CompileStage, Error, WorkerId};
use sodigy::Command;
use sodigy_fs_api::{WriteMode, join, write_string};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub command: SimpleCommand,
    pub has_error: bool,

    // It's a timestamp (microseconds) since the worker's born.
    // Clocks between workers are not synchronized, but I don't think
    // that'd be a big deal.
    pub start: u64,
    pub end: u64,
}

// TODO: split InterHir, InterMir and Bytecode into finer stages
#[derive(Clone, Debug)]
pub enum SimpleCommand {
    Hir(String),
    InterHir,
    Mir(String),
    InterMir,
    PostMir(String),
    Bytecode,
}

impl From<&Command> for SimpleCommand {
    fn from(c: &Command) -> SimpleCommand {
        match c {
            Command::PerFileIr { input_module_path, stop_after, .. } => match stop_after {
                CompileStage::Hir => SimpleCommand::Hir(input_module_path.to_string()),
                CompileStage::Mir => SimpleCommand::Mir(input_module_path.to_string()),
                CompileStage::PostMir => SimpleCommand::PostMir(input_module_path.to_string()),
                _ => unreachable!(),
            },
            Command::InterHir { .. } => SimpleCommand::InterHir,
            Command::InterMir { .. } => SimpleCommand::InterMir,
            Command::Bytecode { .. } => SimpleCommand::Bytecode,
        }
    }
}

struct Whole {
    pub hir: Option<(u64, u64)>,
    pub inter_hir: Option<(u64, u64)>,
    pub mir: Option<(u64, u64)>,
    pub inter_mir: Option<(u64, u64)>,
    pub post_mir: Option<(u64, u64)>,
    pub bytecode: Option<(u64, u64)>,
}

pub fn dump_log(
    mut worker_ids: Vec<WorkerId>,
    logs: &HashMap<WorkerId, Vec<LogEntry>>,
    ir_dir: &str,
) -> Result<(), Error> {
    let mut result = vec![];
    worker_ids.sort();
    let mut first_timestamp = u64::MAX;
    let mut last_timestamp = 0;
    let mut whole_timestamps = Whole {
        hir: None,
        inter_hir: None,
        mir: None,
        inter_mir: None,
        post_mir: None,
        bytecode: None,
    };

    let logs: Vec<(WorkerId, Option<Vec<LogEntry>>)> = worker_ids.iter().map(
        |id| (
            *id,
            logs.get(id).map(
                |log| log.iter().map(
                    |log| {
                        let LogEntry { command, start, end, .. } = log;
                        first_timestamp = first_timestamp.min(*start);
                        last_timestamp = last_timestamp.max(*end);

                        match &command {
                            SimpleCommand::Hir(_) => match &mut whole_timestamps.hir {
                                Some((h_start, h_end)) => {
                                    *h_start = (*h_start).min(*start);
                                    *h_end = (*h_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.hir = Some((*start, *end));
                                },
                            },
                            SimpleCommand::InterHir => match &mut whole_timestamps.inter_hir {
                                Some((h_start, h_end)) => {
                                    *h_start = (*h_start).min(*start);
                                    *h_end = (*h_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.inter_hir = Some((*start, *end));
                                },
                            },
                            SimpleCommand::Mir(_) => match &mut whole_timestamps.mir {
                                Some((m_start, m_end)) => {
                                    *m_start = (*m_start).min(*start);
                                    *m_end = (*m_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.mir = Some((*start, *end));
                                },
                            },
                            SimpleCommand::InterMir => match &mut whole_timestamps.inter_mir {
                                Some((m_start, m_end)) => {
                                    *m_start = (*m_start).min(*start);
                                    *m_end = (*m_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.inter_mir = Some((*start, *end));
                                },
                            },
                            SimpleCommand::PostMir(_) => match &mut whole_timestamps.post_mir {
                                Some((m_start, m_end)) => {
                                    *m_start = (*m_start).min(*start);
                                    *m_end = (*m_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.post_mir = Some((*start, *end));
                                },
                            },
                            SimpleCommand::Bytecode => match &mut whole_timestamps.bytecode {
                                Some((b_start, b_end)) => {
                                    *b_start = (*b_start).min(*start);
                                    *b_end = (*b_end).max(*end);
                                },
                                None => {
                                    whole_timestamps.bytecode = Some((*start, *end));
                                },
                            },
                        }

                        log.clone()
                    }
                ).collect::<Vec<_>>()
            ),
        )
    ).collect();

    result.push(String::from("# Timings

*Warning*: Turn off incremental compilation to get accurate result.
Hir stages are likely to run much faster if incremental compilation is enabled.

## Timings per stage

This section shows how much time compiler spends at each stage.
"));

    if let Some((start, end)) = whole_timestamps.hir {
        result.push(format!("- hir: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    if let Some((start, end)) = whole_timestamps.inter_hir {
        result.push(format!("- inter_hir: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    if let Some((start, end)) = whole_timestamps.mir {
        result.push(format!("- mir: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    if let Some((start, end)) = whole_timestamps.inter_mir {
        result.push(format!("- inter_mir: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    if let Some((start, end)) = whole_timestamps.post_mir {
        result.push(format!("- post_mir: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    if let Some((start, end)) = whole_timestamps.bytecode {
        result.push(format!("- bytecode: {} ({})", prettify_micros(end - start), percentage(end - start, last_timestamp - first_timestamp)));
    }

    result.push(String::from("
## Timings per file

It shows what file each worker is working on at each timing. If there're no space, I wrote a number in a parenthesis and
wrote the file name in footnotes.
"));

    let width = 192;
    let hir_entries = logs.iter().map(
        |(worker_id, entries)| (
            *worker_id,
            entries.as_ref().map(|entries| entries.iter().filter_map(
                |entry| match entry {
                    LogEntry {
                        command: SimpleCommand::Hir(f),
                        has_error,
                        start,
                        end,
                    } => Some((format!("{f}{}", if *has_error { "!" } else { "" }), *start, *end)),
                    _ => None,
                }
            ).collect::<Vec<(String, u64, u64)>>()),
        )
    ).collect::<Vec<_>>();

    if hir_entries.iter().any(
        |(_, entries)| match entries {
            Some(v) if !v.is_empty() => true,
            _ => false,
        }
    ) {
        result.push(format!("### hir\n\n```\n{}\n```\n\n", draw_per_worker_graph(hir_entries, width)));
    }

    let mir_entries = logs.iter().map(
        |(worker_id, entries)| (
            *worker_id,
            entries.as_ref().map(|entries| entries.iter().filter_map(
                |entry| match entry {
                    LogEntry {
                        command: SimpleCommand::Mir(f),
                        has_error,
                        start,
                        end,
                    } => Some((format!("{f}{}", if *has_error { "!" } else { "" }), *start, *end)),
                    _ => None,
                }
            ).collect::<Vec<(String, u64, u64)>>()),
        )
    ).collect::<Vec<_>>();

    if mir_entries.iter().any(
        |(_, entries)| match entries {
            Some(v) if !v.is_empty() => true,
            _ => false,
        }
    ) {
        result.push(format!("### mir\n\n```\n{}\n```\n\n", draw_per_worker_graph(mir_entries, width)));
    }

    let post_mir_entries = logs.iter().map(
        |(worker_id, entries)| (
            *worker_id,
            entries.as_ref().map(|entries| entries.iter().filter_map(
                |entry| match entry {
                    LogEntry {
                        command: SimpleCommand::PostMir(f),
                        has_error,
                        start,
                        end,
                    } => Some((format!("{f}{}", if *has_error { "!" } else { "" }), *start, *end)),
                    _ => None,
                }
            ).collect::<Vec<(String, u64, u64)>>()),
        )
    ).collect::<Vec<_>>();

    if post_mir_entries.iter().any(
        |(_, entries)| match entries {
            Some(v) if !v.is_empty() => true,
            _ => false,
        }
    ) {
        result.push(format!("### post_mir\n\n```\n{}\n```\n\n", draw_per_worker_graph(post_mir_entries, width)));
    }

    write_string(
        &join(ir_dir, "timings")?,
        &result.join("\n"),
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

fn prettify_micros(us: u64) -> String {
    match us {
        ..100_000 => format!("{}.{:03} ms", us / 1000, us % 1000),
        ..60_000_000 => format!("{}.{:03} sec", us / 1_000_000, us / 1000 % 1000),
        _ => format!("{} min {} sec", us / 60_000_000, us / 1_000_000 % 60),
    }
}

fn percentage(numer: u64, denom: u64) -> String {
    let permil = numer as u128 * 1000 / denom as u128;
    format!("{}.{}%", permil / 10, permil % 10)
}

fn draw_per_worker_graph(
    graph: Vec<(WorkerId, Option<Vec<(String, u64, u64)>>)>,
    total_width: usize,
) -> String {
    let graph_width = total_width - 12;  // 12 characters for y_labels
    let mut min_time = u64::MAX;
    let mut max_time = 0;
    let mut has_entry = false;

    for (_, entries) in graph.iter() {
        if let Some(entries) = entries {
            for (_, start, end) in entries.iter() {
                has_entry = true;
                min_time = min_time.min(*start);
                max_time = max_time.max(*end);
            }
        }
    }

    let mut axis = vec![vec![' '; total_width + 9]; 2];  // 9 characters for the last x_label (just in case)

    if has_entry {
        for x in 12..total_width {
            axis[1][x] = '-';
        }

        for i in 0..9u64 {
            if total_width < 160 && i % 2 == 1 {
                continue;
            }

            let x = 12 + (graph_width as u64 * i / 8).min(graph_width as u64 - 1);
            let label = (min_time * (8 - i) + max_time * i) / 8;
            let label: Vec<char> = prettify_micros(label).chars().collect();
            axis[1][x as usize] = '*';

            for (i, c) in label.iter().enumerate() {
                axis[0][x as usize - label.len() / 2 + i as usize] = *c;
            }
        }
    }

    let axis = axis.iter().map(|line| line.iter().collect::<String>()).collect::<Vec<_>>().join("\n");
    let mut shorten_label_index = 0;
    let mut shorten_labels: Vec<(u32, String)> = vec![];
    let mut omitted_labels = vec![];

    let graph = graph.iter().map(
        |(id, entries)| {
            let worker = format!("worker {:>3}", id.0);

            match entries {
                Some(entries) => {
                    let mut occupied = vec![None; graph_width];
                    let mut rendered = vec![' '; graph_width];

                    for (file, start, end) in entries.iter() {
                        let start = (start - min_time) * graph_width as u64 / (max_time - min_time);
                        let end = (end - min_time) * graph_width as u64 / (max_time - min_time);

                        for i in start..end {
                            occupied[i as usize] = Some(file.to_string());
                        }
                    }

                    let mut cursor = 0;
                    let mut curr_file_start;
                    let mut curr_file;

                    'entries: loop {
                        'entry: loop {
                            match occupied.get(cursor) {
                                Some(None) => {
                                    cursor += 1;
                                },
                                Some(Some(file)) => {
                                    curr_file_start = cursor;
                                    curr_file = file.to_string();
                                    break 'entry;
                                },
                                None => {
                                    break 'entries;
                                },
                            }
                        }

                        'entry: loop {
                            match occupied.get(cursor) {
                                Some(Some(file)) if file == &curr_file => {
                                    cursor += 1;
                                },
                                _ => {
                                    break 'entry;
                                },
                            }
                        }

                        let shorten_label = format!("({shorten_label_index})");

                        if cursor - curr_file_start > shorten_label.len() + 2 {
                            let label = if cursor - curr_file_start > curr_file.chars().count() + 2 {
                                curr_file.to_string()
                            } else {
                                shorten_labels.push((shorten_label_index, curr_file.to_string()));
                                shorten_label_index += 1;
                                shorten_label.to_string()
                            };

                            rendered[curr_file_start] = '<';
                            rendered[cursor - 1] = '>';

                            for i in (curr_file_start + 1)..(cursor - 1) {
                                rendered[i] = '-';
                            }

                            for (i, c) in label.chars().enumerate() {
                                rendered[curr_file_start + (cursor - curr_file_start) / 2 - label.len() / 2 + i] = c;
                            }
                        }

                        else {
                            for i in curr_file_start..cursor {
                                rendered[i] = '.';
                            }

                            omitted_labels.push(curr_file.to_string());
                        }
                    }

                    format!("{worker}: {}", rendered.iter().collect::<String>())
                },
                None => format!("{worker}: ???"),
            }
        }
    ).collect::<Vec<_>>().join("\n");
    let shorten_labels = if shorten_labels.is_empty() {
        String::new()
    } else {
        format!("\n{}", shorten_labels.iter().map(
            |(index, label)| format!("({index}): {label}")
        ).collect::<Vec<_>>().join("\n"))
    };
    let omitted_labels = if omitted_labels.is_empty() {
        String::new()
    } else {
        format!("\nomitted files: {}", omitted_labels.join(", "))
    };

    format!("{axis}\n\n{graph}{shorten_labels}{omitted_labels}")
}
