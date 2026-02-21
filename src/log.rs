use crate::{Error, Worker, WorkerId};
use sodigy_driver::CompileStage;
use sodigy_fs_api::{WriteMode, join, write_string};
use std::collections::{HashMap, HashSet};
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

    pub fn log_end(&mut self, has_error: bool) {
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

            self.timings_log.push(LogEntry {
                stage,
                module,
                start,
                end: timestamp,
                has_error: self.curr_stage_error | has_error,
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
const ERROR_MARK: &'static str = r#"<span class="color-red error-mark">!</span>"#;

// TODO: test with empty timings

// VIBE NOTE: I don't know much about html/css, so GEMINI and KIMI-K2.5 (both via Perplexity) did a lot of work.
//            They only did the html/css part.
fn dump_timings_html(
    worker_ids: &[WorkerId],
    timings: &HashMap<WorkerId, Vec<LogEntry>>,
) -> String {
    // TODO: adjust canvas_size when the canvas is too big/small
    let canvas_size = 4096;  // in pixels

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
    let mut longest_stage_frames = 0;
    let mut shortest_stage_frames = usize::MAX;
    let mut longest_stage = None;
    let mut shortest_stage = None;
    let mut total_stages = 0;
    let mut all_modules = HashSet::new();

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

                    total_stages += 1;

                    if let Some(module) = &entry.module {
                        all_modules.insert(module.to_string());
                    }

                    if frame_end - frame_start > longest_stage_frames {
                        longest_stage_frames = frame_end - frame_start;
                        longest_stage = Some(entry.clone());
                    }

                    if frame_end - frame_start < shortest_stage_frames {
                        shortest_stage_frames = frame_end - frame_start;
                        shortest_stage = Some(entry.clone());
                    }
                }
            },
            None => {
                erroneous_workers.push(worker_id.0);
            },
        }
    }

    let mut curr_stage = vec![None; worker_ids.len()];
    let mut frame_per_stage: HashMap<CompileStage, usize> = HashMap::new();
    let mut total_frames = 0;

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
                total_frames += 1;
            }
        }
    }

    let longest_stage_str = if let Some(longest_stage) = &longest_stage {
        format!(
            r#"<li>longest stage: <span class="legend {:?}">{:?}</span>{}, took {}</li>"#,
            longest_stage.stage,
            longest_stage.stage,
            if let Some(module) = &longest_stage.module { format!(" ({module})") } else { String::new() },
            render_micro_seconds(longest_stage_frames as u64 * total_elapsed_us / FRAME_COUNT as u64),
        )
    } else {
        String::new()
    };

    let stats = format!(r#"
<ul>
    <li>elapsed time: {}</li>
    <li>total workers: {}</li>
    <li>total modules: {}</li>
    <li>total stages: {total_stages}</li>
    {longest_stage_str}
</ul>
"#,
        render_micro_seconds(total_elapsed_us),
        worker_ids.len(),
        all_modules.len(),
    );

    let legend = {
        let mut elements = vec![];
        let mut compile_stages = frame_per_stage.keys().collect::<Vec<_>>();
        compile_stages.sort();

        for compile_stage in compile_stages.iter() {
            let frames = *frame_per_stage.get(compile_stage).unwrap();
            let percentage = (frames * 100_000 / total_frames) as f64 / 1000.0;
            elements.push(format!(r#"<li><span class="legend {compile_stage:?}">{compile_stage:?}</span>: {percentage:.2}%</li>"#));
        }

        let elements = elements.concat();
        format!("<ul>{elements}</ul>")
    };

    let x_labels = {
        let mut labels = vec![];
        let label_count = (canvas_size / 256).max(4);

        for i in 1..label_count {
            let label = i * total_elapsed_us as usize / label_count + start_min as usize;
            let label = format!("{:.2}ms", label as f64 / 1000.0);
            let left = i * canvas_size / label_count;
            labels.push(format!(r#"<span class="x-label" style="left: {}px;">{label}</span>"#, left - 30));
            labels.push(format!(r#"<span class="x-label-marker" style="left: {left}px;"></span>"#));
        }

        labels.concat()
    };

    let y_labels = {
        let mut rows = vec![];

        // empty label for x-labels
        rows.push(String::from(r#"<div class="graph-row graph-row-label"></div>"#));

        for (worker_id, _) in frames.iter().enumerate() {
            rows.push(format!(
                r#"<div class="graph-row graph-row-label">Worker-{worker_id}{}</div>"#,
                if erroneous_workers.contains(&worker_id) { ERROR_MARK } else { "" },
            ));
        }

        rows.concat()
    };

    let rows = {
        let mut rows = vec![];
        rows.push(format!(r#"<div class="graph-row graph-row-blocks"><span id="x-labels">{x_labels}</span></div>"#));

        for frames in frames.iter() {
            let mut curr_block = None;
            let mut blocks = vec![];

            for (i, frame) in frames.iter().enumerate() {
                match frame {
                    Frame::New(_) | Frame::Empty => {
                        if let Some(block) = curr_block {
                            blocks.push(generate_block(&block, i, canvas_size));
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
                blocks.push(generate_block(&block, FRAME_COUNT, canvas_size));
            }

            let blocks = blocks.concat();
            rows.push(format!(r#"<div class="graph-row graph-row-blocks" style="width: {canvas_size}px;">{blocks}</div>"#));
        }

        rows.concat()
    };

    let style = r#"
html {
    background-color: rgb(32, 32, 32);
    color: rgb(255, 255, 255);
    scrollbar-color: rgb(255, 255, 255) rgb(32, 32, 32);
}

#graph-canvas {
    display: flex;
}

#graph-labels-column {
    flex-shrink: 0;
    width: 160px;
}

#graph-rows-column {
    flex-grow: 1;
    overflow-x: auto;
}

#graph-rows-wrapper {
    position: relative;
}

.graph-row {
    box-sizing: border-box;
    height: 30px;
}

.graph-column .graph-row:nth-child(1) {
    height: 120px;
    border: none;
}

.graph-column .graph-row:nth-child(even) {
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
    width: 160px;
    text-align: right;
    padding-top: 4px;
    padding-bottom: 4px;
    padding-right: 10px;
    border-right: 4px solid rgb(255, 255, 255);
}

.graph-row-blocks {
    position: relative;
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

.x-label {
    display: inline-block;
    position: absolute;
    text-align: center;
    width: 60px;
    bottom: 40px;
}

.x-label-marker {
    display: inline-block;
    position: absolute;
    width: 4px;
    height: 20px;
    background-color: rgb(255, 255, 255);
    bottom: 5px;
}

.tooltip {
    display: block;
    visibility: hidden;
    background: rgb(255, 255, 255);
    color: rgb(0, 0, 0);
    padding: 6px;
    bottom: 125%;
    left: 50%;
    border-radius: 6px;
    position: absolute;
    z-index: 100;
}

.graph-block:hover .tooltip {
    visibility: visible;
}

.error-mark {
    padding-left: 8px;
    padding-right: 8px;
    margin-left: 4px;
    border-radius: 50px;
    background: rgb(255, 255, 255);
}

.Load {
    background-color: rgba(160, 160, 160, 0.5);
}

.Load:hover {
    background-color: rgba(160, 160, 160, 1.0);
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
    background-color: rgba(224, 128, 32, 0.5);
}

.Mir:hover {
    background-color: rgba(224, 128, 32, 1.0);
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

.MirOptimize {
    background-color: rgba(128, 208, 32, 0.5);
}

.MirOptimize:hover {
    background-color: rgba(128, 208, 32, 1.0);
}

.Bytecode {
    background-color: rgba(128, 96, 208, 0.5);
}

.Bytecode:hover {
    background-color: rgba(128, 96, 208, 1.0);
}

.BytecodeOptimize {
    background-color: rgba(32, 208, 128, 0.5);
}

.BytecodeOptimize:hover {
    background-color: rgba(32, 208, 128, 1.0);
}

.CodeGen {
    background-color: rgba(208, 192, 64, 0.5);
}

.CodeGen:hover {
    background-color: rgba(208, 192, 64, 1.0);
}

.color-red {
    color: rgb(192, 32, 32);
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
<h2>Stats</h2>
<div id="stats">{stats}</div>
<h2>Stages</h2>
<div id="legend">{legend}</div>
<p>
It doesn't measure the elapsed time of each stage because it's too difficult to do so.
There are multiple modules, multiple workers and multiple stages. Some stages are parallel.
</p>
<p>
If you don't see lex, parse and hir stages, it's likely because incremental compilation is enabled.
</p>
<h2>Timings</h2>
<div id="graph">
    <div id="graph-canvas">
        <div id="graph-labels-column" class="graph-column">{y_labels}</div>
        <div id="graph-rows-column" class="graph-column">
            <div id="graph-rows-wrapper">{rows}</div>
        </div>
    </div>
</div>
</body>
</html>"#)
}

fn generate_block((entry, start): &(&LogEntry, usize), end: usize, canvas_size: usize) -> String {
    let tooltip_message = format!(
        "{:?}{}<br/>({:.2}ms){}",
        entry.stage,
        if let Some(module) = &entry.module { format!("<br/>{module}") } else { String::new() },
        (entry.end - entry.start) as f64 / 1000.0,
        if entry.has_error { r#"<br/><span class="color-red">has error</span>"# } else { "" },
    );
    let tooltip_style = if (*start + end) < FRAME_COUNT / 64 {
        // right-align (default)
        ""
    } else {
        // center-align
        "transform: translateX(-50%);"
    };

    let tooltip_container = format!(r#"<span class="tooltip" style="{tooltip_style}">{tooltip_message}</span>"#);

    let width = (end - start) * canvas_size / FRAME_COUNT;
    let left = start * canvas_size / FRAME_COUNT;
    let long_title = format!(
        "{:?}{}{}",
        entry.stage,
        if let Some(module) = &entry.module { format!(" ({module})") } else { String::new() },
        if entry.has_error { ERROR_MARK } else { "" },
    );

    let title = if width > long_title.len() * 8 {
        long_title
    } else if width > 80 {
        format!("{:?}{}", entry.stage, if entry.has_error { ERROR_MARK } else { "" })
    } else if width > 20 && entry.has_error {
        ERROR_MARK.to_string()
    } else {
        String::new()
    };

    format!(
        r#"<span class="graph-block {:?}" style="width: {width}px; left: {left}px;">{title}{tooltip_container}</span>"#,
        entry.stage,
    )
}

fn render_micro_seconds(us: u64) -> String {
    if us < 100_000 {
        format!("{:.2}ms", us as f64 / 1000.0)
    } else {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    }
}
