use crate::{CompileStage, Error, Worker, WorkerId};
use sodigy_fs_api::{WriteMode, join, write_string};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub stage: CompileStage,
    pub module: Option<String>,
    pub has_error: bool,

    // It's a timestamp (microseconds) since the worker's birth.
    // Clocks between workers are not synchronized, but I don't think
    // that'd be a big deal.
    pub start: u64,
    pub end: u64,
}

impl Worker {
    pub fn log_start(&mut self, stage: CompileStage, module: Option<String>) {
        assert!(self.curr_stage.is_none());
        let timestamp = Instant::now().duration_since(self.born_at.clone()).as_micros() as u64;

        if let Some(file) = &self.log_file {
            if let Err(e) = write_string(
                file,
                &format!("[Worker-{}][timestamp: {} ms] start {:?}\n", self.id.0, timestamp / 1000, (stage, &module)),
                WriteMode::AppendOrCreate,
            ) {
                eprintln!("Error while writing log (worker-{}): {e:?}", self.id.0);
            }
        }

        self.curr_stage = Some((stage, module, timestamp));
        self.curr_stage_error = false;
    }

    pub fn log_end(&mut self) {
        if let Some((stage, module, start)) = self.curr_stage.take() {
            let timestamp = Instant::now().duration_since(self.born_at.clone()).as_micros() as u64;

            if let Some(file) = &self.log_file {
                if let Err(e) = write_string(
                    file,
                    &format!("[Worker-{}][timestamp: {} ms] end {:?}\n", self.id.0, timestamp / 1000, (stage, &module)),
                    WriteMode::AppendOrCreate,
                ) {
                    eprintln!("Error while writing log (worker-{}): {e:?}", self.id.0);
                }
            }

            self.history.push(LogEntry {
                stage,
                module,
                start,
                end: timestamp,
                has_error: self.curr_stage_error,
            });
        }
    }
}

pub fn dump_timings(
    mut worker_ids: Vec<WorkerId>,
    timings: &HashMap<WorkerId, Vec<LogEntry>>,
    ir_dir: &str,
) -> Result<(), Error> {
    worker_ids.sort();

    let json = dump_timings_json(&worker_ids, timings);
    write_string(
        &join(ir_dir, "timings.json")?,
        &json,
        WriteMode::CreateOrTruncate,
    )?;

    let html = dump_timings_html(&worker_ids, timings);
    write_string(
        &join(ir_dir, "timings.html")?,
        &html,
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

fn dump_timings_json(worker_ids: &[WorkerId], timings: &HashMap<WorkerId, Vec<LogEntry>>) -> String {
    // I know that you want to depend on serde_json... but please don't!
    // I want sodigy to strictly follow No-Dependency-Rule (every code has to be within this repository),
    // and I don't want to break the rule for just this function.
    let mut lines = vec![];
    lines.push(format!("{{"));
    lines.push(format!("    \"worker_ids\": {:?},", worker_ids.iter().map(|id| id.0).collect::<Vec<_>>()));
    lines.push(format!("    \"timings\": {{"));

    for (i, (worker_id, entries)) in timings.iter().enumerate() {
        lines.push(format!("        \"{}\": [", worker_id.0));

        for (j, entry) in entries.iter().enumerate() {
            lines.push(format!("            {{"));
            lines.push(format!("                \"stage\": {:?},", format!("{:?}", entry.stage)));
            lines.push(format!("                \"module\": {},", if let Some(module) = &entry.module { format!("{module:?}") } else { String::from("null") }));
            lines.push(format!("                \"has_error\": {},", entry.has_error));
            lines.push(format!("                \"start\": {},", entry.start));
            lines.push(format!("                \"end\": {}", entry.end));
            lines.push(format!("            }}{}", if j + 1 == entries.len() { "" } else { "," }));
        }

        lines.push(format!("        ]{}", if i + 1 == timings.len() { "" } else { "," }));
    }

    lines.push(format!("    }}"));
    lines.push(format!("}}"));
    lines.join("\n")
}

// We'll use `Vec<Frame>` data structure, which is easier to render.
// It's like Adobe Flash animations!
#[derive(Clone)]
pub enum Frame {
    Empty,
    New(LogEntry),
    Same,  // as previous frame
}

const FRAME_COUNT: usize = 4096;

// TODO: a lot more styling
// TODO: hover effects
// TODO: test with empty timings

// VIBE NOTE: I don't know much about html/css, so GEMINI and KIMI-K2.5 (both via Perplexity) did a lot of work.
//            They only did the html/css part.
fn dump_timings_html(worker_ids: &[WorkerId], timings: &HashMap<WorkerId, Vec<LogEntry>>) -> String {
    let mut frames = vec![vec![Frame::Empty; FRAME_COUNT]; worker_ids.len()];
    let mut start_min = u64::MAX;
    let mut end_max = 0;

    for entries in timings.values() {
        for entry in entries.iter() {
            start_min = start_min.min(entry.start);
            end_max = end_max.max(entry.end);
        }
    }

    let total_elapsed_us = end_max - start_min;

    // If a worker crashes (likely due to an internal compiler error), there's no
    // entry in `timings`.
    let mut erroneous_workers = vec![];

    for worker_id in worker_ids.iter() {
        match timings.get(worker_id) {
            Some(entries) => {
                for entry in entries.iter() {
                    let frame_start = (entry.start - start_min) as usize * FRAME_COUNT / (end_max - start_min) as usize;
                    let frame_end = (entry.end - start_min) as usize * FRAME_COUNT / (end_max - start_min) as usize;
                    frames[worker_id.0][frame_start] = Frame::New(entry.clone());

                    for i in (frame_start + 1)..frame_end {
                        frames[worker_id.0][i] = Frame::Same;
                    }
                }
            },
            None => {
                erroneous_workers.push(worker_id.0);
            },
        }
    }

    // TODO: we have to count in different way
    //       just count the number of frames per stage, then divide the parallel stages by the number of workers
    let mut curr_stage = vec![None; worker_ids.len()];
    let mut frame_per_stage: HashMap<CompileStage, usize> = HashMap::new();

    for frame in 0..FRAME_COUNT {
        for worker_id in worker_ids.iter() {
            match &frames[worker_id.0][frame] {
                Frame::Empty => {
                    curr_stage[worker_id.0] = None;
                },
                Frame::New(e) => {
                    curr_stage[worker_id.0] = Some(e.stage);
                },
                Frame::Same => {},
            }
        }

        for stage in curr_stage.iter() {
            if let Some(stage) = stage {
                frame_per_stage.insert(*stage, *frame_per_stage.get(stage).unwrap_or(&0) + 1);
                break;
            }
        }
    }

    let legend = {
        let mut elements = vec![];
        let mut compile_stages = frame_per_stage.keys().collect::<Vec<_>>();
        compile_stages.sort();

        for compile_stage in compile_stages.iter() {
            let frames = *frame_per_stage.get(compile_stage).unwrap();
            let ms = (frames * total_elapsed_us as usize / FRAME_COUNT) as f64 / 1000.0;
            let percentage = (frames * 100_000 / FRAME_COUNT) as f64 / 1000.0;
            elements.push(format!(
                r#"<li><span class="legend {}">{}</span>: {ms:.2}ms ({percentage:.2}%)</li>"#,
                format!("{compile_stage:?}"),
                format!("{compile_stage:?}"),
            ));
        }

        let elements = elements.concat();
        format!("<ul>{elements}</ul>")
    };

    let labels = {
        let mut rows = vec![];

        for (worker_id, _) in frames.iter().enumerate() {
            rows.push(format!(r#"<div class="graph-row graph-row-label">Worker-{worker_id}</div>"#));
        }

        rows.concat()
    };

    let rows = {
        let mut rows = vec![];

        for frames in frames.iter() {
            let mut curr_block = None;
            let mut blocks = vec![];

            for (i, frame) in frames.iter().enumerate() {
                match frame {
                    Frame::New(_) | Frame::Empty => {
                        if let Some(block) = curr_block {
                            blocks.push(generate_block(&block, i));
                        }

                        curr_block = None;

                        if let Frame::New(e) = frame {
                            curr_block = Some((e, i));
                        }
                    },
                    Frame::Same => {},
                }
            }

            if let Some(block) = curr_block {
                blocks.push(generate_block(&block, FRAME_COUNT));
            }

            let blocks = blocks.concat();
            rows.push(format!(r#"<div class="graph-row graph-row-blocks">{blocks}</div>"#));
        }

        rows.concat()
    };

    let style = r#"
html {
    background-color: rgb(32, 32, 32);
    color: rgb(255, 255, 255);
}

#graph-canvas {
    display: flex;
}

#graph-labels-column {
    flex-shrink: 0;
    width: 100px;
}

#graph-rows-column {
    flex-grow: 1;
    overflow-x: auto;
    overflow-y: hidden;
}

#graph-rows-wrapper {
    position: relative;
}

.graph-column .graph-row:nth-child(odd) {
    background-color: rgb(72, 72, 72);
}

.legend {
    padding: 4px;
    border-radius: 12px;
}

#legend li {
    padding-top: 10px;
}

.graph-row-label {
    display: inline-block;
    width: 100px;
    height: 30px;
    text-align: right;
    padding-right: 10px;
}

.graph-row-blocks {
    position: relative;
    height: 30px;
}

.graph-block {
    display: inline-block;
    position: absolute;
    text-align: center;
    border-radius: 12px;
    height: 24px;
    margin-top: 3px;
    margin-bottom: 3px;
}

.Lex {
    background-color: rgba(192, 192, 64, 0.5);
}

.Lex:hover {
    background-color: rgba(192, 192, 64, 1.0);
}

.Parse {
    background-color: rgba(64, 64, 192, 0.5);
}

.Parse:hover {
    background-color: rgba(64, 64, 192, 1.0);
}

.Hir {
    background-color: rgba(64, 192, 64, 0.5);
}

.Hir:hover {
    background-color: rgba(64, 192, 64, 1.0);
}

.InterHir {
    background-color: rgba(192, 64, 192, 0.5);
}

.InterHir:hover {
    background-color: rgba(192, 64, 192, 1.0);
}

.Mir {
    background-color: rgba(128, 128, 128, 0.5);
}

.Mir:hover {
    background-color: rgba(128, 128, 128, 1.0);
}

.InterMir {
    background-color: rgba(192, 64, 64, 0.5);
}

.InterMir:hover {
    background-color: rgba(192, 64, 64, 1.0);
}

.PostMir {
    background-color: rgba(64, 192, 192, 0.5);
}

.PostMir:hover {
    background-color: rgba(64, 192, 192, 1.0);
}
"#;
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Sodigy Compiler Timeline</title>
<style>{style}</style>
</head>
<body>
<div id="legend">
    {legend}
</div>
<div id="graph">
    <div id="graph-title" style="text-align: center; font-size: 24px;">Timings</div>
    <div id="graph-canvas">
        <div id="graph-labels-column" class="graph-column">{labels}</div>
        <div id="graph-rows-column" class="graph-column">
            <div id="graph-rows-wrapper">{rows}</div>
        </div>
    </div>
</div>
</body>
</html>"#)
}

const CANVAS_SIZE: usize = 4096;  // pixels

fn generate_block((entry, start): &(&LogEntry, usize), end: usize) -> String {
    let hover = format!(
        "{}{}<br/>({:.2}ms)",
        format!("{:?}", entry.stage),
        if let Some(module) = &entry.module { format!(" {module}") } else { String::new() },
        (entry.end - entry.start) as f64 / 1000.0,
    );

    let title = if end > start + FRAME_COUNT / 64 {
        format!("{:?}", entry.stage)
    } else {
        String::new()
    };

    let width = (end - start) * CANVAS_SIZE / FRAME_COUNT;
    let left = start * CANVAS_SIZE / FRAME_COUNT;
    format!(
        r#"<span class="graph-block {:?}" style="width: {width}px; left: {left}px;">{title}</span>"#,
        entry.stage,
    )
}
