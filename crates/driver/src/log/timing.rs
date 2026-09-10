use crate::{Error, Worker, WorkerId};
use sodigy_fs_api::{WriteMode, join, write_string};
use sodigy_stages::{Stage, STAGES, Substage};
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
    too_long: u64,
    total_stages: usize,
    total_modules: usize,
    times_all: Vec<(TimingsEntry, u64)>,
    substage_per_stage: HashMap<Stage, HashSet<Substage>>,
    times_per_stage: HashMap<(Stage, Option<Substage>), Vec<(TimingsEntry, u64)>>,
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

    let longest_stages = if stats.times_all.is_empty() {
        String::new()
    } else {
        let longest_stages = if stats.times_all.len() > 5 {
            stats.times_all[..5].to_vec()
        } else {
            stats.times_all.to_vec()
        };
        let longest_stages: Vec<String> = longest_stages.iter().map(
            |(t, s)| format!(
                r#"<li><span class="legend {:?}">{:?}</span>: {}{}{}</li>"#,
                t.stage,
                t.stage,
                if let Some(substage) = t.substage {
                    format!("{}, ", substage.render())
                } else {
                    String::new()
                },
                render_micro_seconds(*s, u64::MAX /* we don't color this */),
                if let Some(module) = &t.module {
                    format!(" ({module})")
                } else {
                    String::new()
                },
            )
        ).collect();
        let longest_stages = longest_stages.concat();

        format!("<li>longest stages<ul>{longest_stages}</ul></li>")
    };

    let stats_str = format!(r#"
<ul>
    <li>elapsed time: {}</li>
    <li>total workers: {}</li>
    <li>total modules: {}</li>
    <li>total stages: {}</li>
    {longest_stages}
</ul>
"#,
        render_micro_seconds(stats.end - stats.start, u64::MAX  /* we don't color this */),
        worker_ids.len(),
        stats.total_modules,
        stats.total_stages,
    );

    let per_stage_stats = {
        // I want a per-stage stats. If a stage has substages, it has to be per-substage stats.
        // 1. top 5 longest modules
        // 2. average elapsed time
        // 3. average of top 5 longest modules
        //
        // Other than the per-stage stats, I want the top 5 longest work.
        let mut buffer = vec![];

        for stage in STAGES.iter() {
            let mut substages: Vec<Substage> = stats.substage_per_stage.get(stage).unwrap_or(&HashSet::new()).iter().map(|s| *s).collect();
            let mut modules = 0;
            let mut longest_stages: Vec<String> = vec![];
            let mut stage_stats = String::new();
            substages.sort();

            if substages.is_empty() {
                let times: Vec<(TimingsEntry, u64)> = stats.times_per_stage.get(&(*stage, None)).unwrap_or(&vec![]).to_vec();
                modules += times.len();

                for (e, t) in times.iter().take(5) {
                    longest_stages.push(format!(
                        "<li>{}: {}</li>",
                        if let Some(module) = &e.module { module } else { "_" },
                        render_micro_seconds(*t, stats.too_long),
                    ));
                }

                if !times.is_empty() {
                    stage_stats = format!(
                        " (avg: {}, min: {}, max: {})",
                        render_micro_seconds((times.iter().map(|(_, t)| *t).sum::<u64>() as f64 / times.len() as f64) as u64, stats.too_long),
                        render_micro_seconds(times.last().unwrap().1, stats.too_long),
                        render_micro_seconds(times.first().unwrap().1, stats.too_long),
                    );
                }
            } else {
                for substage in substages.iter() {
                    let times: Vec<(TimingsEntry, u64)> = stats.times_per_stage.get(&(*stage, Some(*substage))).unwrap_or(&vec![]).to_vec();
                    modules += times.len();

                    if !times.is_empty() {
                        let substage_stats = format!(
                            " (avg: {}, min: {}, max: {})",
                            render_micro_seconds((times.iter().map(|(_, t)| *t).sum::<u64>() as f64 / times.len() as f64) as u64, stats.too_long),
                            render_micro_seconds(times.last().unwrap().1, stats.too_long),
                            render_micro_seconds(times.first().unwrap().1, stats.too_long),
                        );

                        longest_stages.push(format!("<li>{}{substage_stats}<ul>", substage.render()));

                        for (e, t) in times.iter().take(5) {
                            longest_stages.push(format!(
                                "<li>{}: {}</li>",
                                if let Some(module) = &e.module { module } else { "_" },
                                render_micro_seconds(*t, stats.too_long),
                            ));
                        }

                        longest_stages.push(String::from("</ul></li>"));
                    }
                }
            }

            if modules == 0 {
                continue;
            }

            buffer.push(format!(
                r#"
<li>
    <span class="legend {stage:?}">{stage:?}</span>
    <ul>
        <li>modules * substages: {modules}</li>
        <li>substages: {}</li>
        <li>longest stages{stage_stats}<ul>{}</ul></li>
    </ul>
</li>
                "#,
                substages.len(),
                longest_stages.concat(),
            ));
        }

        format!("<ul>{}</ul>", buffer.concat())
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
<div id="per-stage-stats">{per_stage_stats}</div>

<p>
Make sure to check if incremental compilation is enabled!
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
    let mut times_per_stage: HashMap<(Stage, Option<Substage>), Vec<(TimingsEntry, u64)>> = HashMap::new();

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

    times_all.sort_by_key(|(_, t)| u64::MAX - *t);
    let too_long = match times_all.len() {
        // too small sample to threshold
        0..5 => u64::MAX,
        _ => times_all[times_all.len() / 4].1,
    };

    for ts in times_per_stage.values_mut() {
        ts.sort_by_key(|(_, t)| u64::MAX - *t);
    }

    (
        rows,
        Stats {
            start: start_min,
            end: end_max,
            too_long,
            total_stages,
            total_modules: all_modules.len(),
            times_all,
            substage_per_stage,
            times_per_stage,
        },
    )
}

fn render_micro_seconds(us: u64, threshold: u64) -> String {
    let class = if us > threshold { r#" class="color-red""# } else { "" };

    if us < 100_000 {
        format!("<span{class}>{:.2}ms</span>", us as f64 / 1000.0)
    } else {
        format!("<span{class}>{:.2}s</span>", us as f64 / 1_000_000.0)
    }
}
