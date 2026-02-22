use crate::{Error, Worker, WorkerId};
use sodigy_driver::CompileStage;
use sodigy_fs_api::{WriteMode, join, write_string};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

mod graph;

use graph::render_graph;

#[derive(Clone, Debug)]
pub struct TimingsEntry {
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
                    &format!("[Worker-{}][timestamp: {} ms] end {:?}{}\n", self.id.0, timestamp / 1000, (stage, &module), if self.curr_stage_error { " (has_error)" } else { "" }),
                    WriteMode::AppendOrCreate,
                ) {
                    eprintln!("Error while writing log (worker-{}): {e:?}", self.id.0);
                }
            }

            self.timings_log.push(TimingsEntry {
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
    timings: &HashMap<WorkerId, Vec<TimingsEntry>>,
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

fn dump_timings_json(worker_ids: &[WorkerId], timings: &HashMap<WorkerId, Vec<TimingsEntry>>) -> String {
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
pub struct Row {
    pub id: String,
    pub frames: Vec<Frame>,
    pub has_error: bool,
}

impl Row {
    pub fn new(id: String, frames: usize) -> Row {
        Row {
            id,
            frames: vec![Frame::Empty; frames],
            has_error: false,
        }
    }
}

#[derive(Clone)]
pub enum Frame {
    Empty,
    New(TimingsEntry),
    Same,  // as previous frame
}

const FRAME_COUNT: usize = 4096;

struct Stats {
    start: u64,
    end: u64,
    longest_stage_frames: usize,
    longest_stage: Option<TimingsEntry>,
    shortest_stage_frames: usize,
    shortest_stage: Option<TimingsEntry>,
    total_stages: usize,
    total_modules: usize,
}

// TODO: test with empty timings

// VIBE NOTE: I don't know much about html/css, so GEMINI and KIMI-K2.5 (both via Perplexity) did a lot of work.
//            They only did the html/css part.
fn dump_timings_html(
    // It assumes that worker_id starts at 0 and is contiguous.
    worker_ids: &[WorkerId],
    timings: &HashMap<WorkerId, Vec<TimingsEntry>>,
) -> String {
    let (rows, stats) = into_rows(None, worker_ids, timings);
    let mut curr_stage = vec![None; worker_ids.len()];
    let mut frames_per_stage: HashMap<CompileStage, usize> = HashMap::new();
    let mut total_frames = 0;

    for frame in 0..FRAME_COUNT {
        for (i, row) in rows.iter().enumerate() {
            match &row.frames[frame] {
                Frame::Empty => {
                    curr_stage[i] = None;
                },
                Frame::New(e) => {
                    curr_stage[i] = Some(e.stage);
                },
                Frame::Same => {},
            }
        }

        for stage in curr_stage.iter() {
            if let Some(stage) = stage {
                frames_per_stage.insert(*stage, *frames_per_stage.get(stage).unwrap_or(&0) + 1);
                total_frames += 1;
            }
        }
    }

    let longest_stage_str = if let Some(longest_stage) = &stats.longest_stage {
        format!(
            r#"<li>longest stage: <span class="legend {:?}">{:?}</span>{}, took {}</li>"#,
            longest_stage.stage,
            longest_stage.stage,
            if let Some(module) = &longest_stage.module { format!(" ({module})") } else { String::new() },
            render_micro_seconds(stats.longest_stage_frames as u64 * (stats.end - stats.start) / FRAME_COUNT as u64),
        )
    } else {
        String::new()
    };

    let stats_str = format!(r#"
<ul>
    <li>elapsed time: {}</li>
    <li>total workers: {}</li>
    <li>total modules: {}</li>
    <li>total stages: {}</li>
    {longest_stage_str}
</ul>
"#,
        render_micro_seconds(stats.end - stats.start),
        worker_ids.len(),
        stats.total_modules,
        stats.total_stages,
    );

    let legend = {
        let mut elements = vec![];
        let mut compile_stages = frames_per_stage.keys().collect::<Vec<_>>();
        compile_stages.sort();

        for compile_stage in compile_stages.iter() {
            let frames = *frames_per_stage.get(compile_stage).unwrap();
            let percentage = (frames * 100_000 / total_frames) as f64 / 1000.0;
            elements.push(format!(r#"<li><span class="legend {compile_stage:?}">{compile_stage:?}</span>: {percentage:.2}%</li>"#));
        }

        let elements = elements.concat();
        format!("<ul>{elements}</ul>")
    };

    let style = include_str!("timings/style.css");

    // It draws 8 graphs.
    // All stages, long (4096 pixels)
    // All stages, short (1024 pixels)
    // Load/Lex/Parse/Hir stages, long
    // Load/Lex/Parse/Hir stages, short
    // Mir stage, long
    // Mir stage, short
    // Post-Mir/Mir-Optimize/Bytecode/Bytecode-Optimize, long
    // Post-Mir/Mir-Optimize/Bytecode/Bytecode-Optimize, short
    let mut graphs = vec![
        render_graph("graph-all-long", &rows, stats.start, stats.end, 4096),
        render_graph("graph-all-short", &rows, stats.start, stats.end, 1024),
    ];
    let mut radio_buttons = vec![];

    for (stages, id) in [
        (vec![CompileStage::Load, CompileStage::Lex, CompileStage::Parse, CompileStage::Hir], "hir"),
        (vec![CompileStage::Mir], "mir"),
        (vec![CompileStage::PostMir, CompileStage::MirOptimize, CompileStage::Bytecode, CompileStage::BytecodeOptimize], "bytecode"),
    ] {
        let (rows, stats) = into_rows(Some(stages), worker_ids, timings);

        if stats.total_modules == 0 {
            continue;
        }

        graphs.push(render_graph(
            &format!("graph-{id}-long"),
            &rows,
            stats.start,
            stats.end,
            4096,
        ));
        graphs.push(render_graph(
            &format!("graph-{id}-short"),
            &rows,
            stats.start,
            stats.end,
            1024,
        ));
        radio_buttons.push(format!(r#"
<input type="radio" id="select-graph-{id}" name="stages" value="{id}">
<label for="select-graph-{id}">{id}</label>
        "#));
    }

    let radio_buttons = radio_buttons.join("\n");
    let radios = format!(r#"
<input type="radio" id="select-graph-long" name="length" value="long" checked>
<label for="select-graph-long">long</label>
<input type="radio" id="select-graph-short" name="length" value="short">
<label for="select-graph-short">short</label>

<br/>

<input type="radio" id="select-graph-all" name="stages" value="all" checked>
<label for="select-graph-all">all</label>
{radio_buttons}
"#);
    let radios_script = r#"<script>
function updateGraph() {
    // Get selected values
    const length = document.querySelector('input[name="length"]:checked')?.value;
    const stage = document.querySelector('input[name="stages"]:checked')?.value;

    // Hide all graphs
    document.querySelectorAll('.graph').forEach(g => g.classList.remove('active'));

    // Show the matching graph if both are selected
    if (length && stage) {
        const target = document.getElementById(`graph-${stage}-${length}`);
        if (target) {
            target.classList.add('active');
        }
    }
}

// Attach listeners to all radio buttons
document.querySelectorAll('input[name="length"], input[name="stages"]').forEach(radio => {
    radio.addEventListener('change', updateGraph);
});

updateGraph();
</script>
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
<div id="stats">{stats_str}</div>
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
{radios}
{}
{radios_script}
</body>
</html>"#,
        graphs.join("\n"),
    )
}

fn into_rows(
    stages: Option<Vec<CompileStage>>,
    worker_ids: &[WorkerId],
    timings: &HashMap<WorkerId, Vec<TimingsEntry>>,
) -> (Vec<Row>, Stats) {
    let mut rows = Vec::with_capacity(worker_ids.len());
    let mut start_min = u64::MAX;
    let mut end_max = 0;
    let mut longest_stage_frames = 0;
    let mut shortest_stage_frames = usize::MAX;
    let mut longest_stage = None;
    let mut shortest_stage = None;
    let mut total_stages = 0;
    let mut all_modules = HashSet::new();

    for entries in timings.values() {
        for entry in entries.iter() {
            if let Some(stages) = &stages && !stages.contains(&entry.stage) {
                continue;
            }

            start_min = start_min.min(entry.start);
            end_max = end_max.max(entry.end);
        }
    }

    for worker_id in worker_ids.iter() {
        let mut row = Row::new(format!("Worker-{}", worker_id.0), FRAME_COUNT);

        match timings.get(worker_id) {
            Some(entries) => {
                for entry in entries.iter() {
                    if let Some(stages) = &stages && !stages.contains(&entry.stage) {
                        continue;
                    }

                    let frame_start = (entry.start - start_min) as usize * FRAME_COUNT / (end_max - start_min) as usize;
                    let frame_end = (entry.end - start_min) as usize * FRAME_COUNT / (end_max - start_min) as usize;

                    if frame_start == FRAME_COUNT {
                        continue;
                    }

                    row.frames[frame_start] = Frame::New(entry.clone());

                    for i in (frame_start + 1)..frame_end {
                        row.frames[i] = Frame::Same;
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
                row.has_error = true;
            },
        }

        rows.push(row);
    }

    (
        rows,
        Stats {
            start: start_min,
            end: end_max,
            longest_stage_frames,
            longest_stage,
            shortest_stage_frames,
            shortest_stage,
            total_stages,
            total_modules: all_modules.len(),
        },
    )
}

fn render_micro_seconds(us: u64) -> String {
    if us < 100_000 {
        format!("{:.2}ms", us as f64 / 1000.0)
    } else {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    }
}
