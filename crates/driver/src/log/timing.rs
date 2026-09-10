use crate::{Error, Worker, WorkerId};
use sodigy_fs_api::{WriteMode, join, write_string};
use sodigy_stages::{Stage, Substage};
use sodigy_timings::TimingsEntry;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;

mod graph;

use graph::render_graph;

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
            lines.push(format!("                \"substage\": {:?},", if let Some(substage) = entry.substage { format!("{:?}", substage.render()) } else { String::from("null") }));
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

#[derive(Clone, Debug)]
struct Stats {
    start: u64,
    end: u64,
    total_stages: usize,
    total_modules: usize,
    times_all: Vec<(TimingsEntry, u64)>,
    substage_per_stage: HashMap<Stage, HashSet<Substage>>,
    times_per_stage: HashMap<Stage, Vec<(TimingsEntry, u64)>>,
}

// VIBE NOTE: I don't know much about html/css, so GEMINI and KIMI-K2.5 (both via Perplexity) did a lot of work.
//            They only did the html/css part.
fn dump_timings_html(
    // It assumes that worker_id starts at 0 and is contiguous.
    worker_ids: &[WorkerId],
    timings: &HashMap<WorkerId, Vec<TimingsEntry>>,
) -> String {
    let (rows, stats) = into_rows(None, worker_ids, timings);

    if stats.total_modules == 0 {
        // TODO: nicer view for empty graphs
        return String::from("There's no timings info.");
    }

    let mut curr_stage = vec![None; worker_ids.len()];
    let mut frames_per_stage: HashMap<Stage, usize> = HashMap::new();
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
        // I want a per-stage stats. If a stage has substages, it has to be per-substage stats.
        // 1. top 5 longest modules
        // 2. average elapsed time
        // 3. average of top 5 longest modules
        //
        // Other than the per-stage stats, I want the top 5 longest work.
        todo!();
    };

    let style = include_str!("timing/style.css");

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
        (vec![Stage::Load, Stage::Lex, Stage::Parse, Stage::Hir], "hir"),
        (vec![Stage::PostHir, Stage::Mir], "mir"),
        (vec![Stage::PostMir, Stage::MirOptimize, Stage::Bytecode, Stage::BytecodeOptimize], "bytecode"),
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
<title>Sodigy Compiler Timings</title>
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
    stages: Option<Vec<Stage>>,
    worker_ids: &[WorkerId],
    timings: &HashMap<WorkerId, Vec<TimingsEntry>>,
) -> (Vec<Row>, Stats) {
    let mut rows = Vec::with_capacity(worker_ids.len());
    let mut start_min = u64::MAX;
    let mut end_max = 0;
    let mut total_stages = 0;
    let mut all_modules = HashSet::new();
    let mut times_all: Vec<(TimingsEntry, u64)> = vec![];

    let mut substage_per_stage: HashMap<Stage, HashSet<Substage>> = HashMap::new();
    let mut times_per_stage: HashMap<(Stage, Option<Substage>), Vec<u64>> = HashMap::new();

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
                    times_all.push((entry.clone(), entry.end - entry.start));

                    if let Some(substage) = entry.substage {
                        match substage_per_stage.entry(entry.stage) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().insert(substage);
                            },
                            Entry::Vacant(e) => {
                                e.insert([substage].into_iter().collect());
                            },
                        }
                    }

                    match times_per_stage.entry((entry.stage, entry.substage)) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().push((entry.clone(), entry.end - entry.start));
                        },
                        Entry::Vacant(e) => {
                            e.insert(vec![(entry.clone(), entry.end - entry.start)]);
                        },
                    }

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
            total_stages,
            total_modules: all_modules.len(),
            times_all,
            substage_per_stage,
            times_per_stage,
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
