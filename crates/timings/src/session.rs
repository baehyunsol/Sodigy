use crate::TimingsEntry;
use sodigy_fs_api::{WriteMode, write_string};
use sodigy_stages::{CompileStage, StageExtra};
use std::time::Instant;

pub struct TimingsSession {
    // It's supposed to be `sodigy_driver::worker::WorkerId`, but we cannot import the type.
    pub worker_id: usize,

    pub born_at: Instant,
    pub timings_log: Vec<TimingsEntry>,
    pub log_file: Option<String>,
    pub curr_stage: Option<(CompileStage, Option<StageExtra>, Option<String>, u64)>,
    pub curr_stage_error: bool,

    // The caller will set this before calling `.stage_start()`.
    pub module: Option<String>,
}

impl TimingsSession {
    pub fn dummy() -> Self {
        TimingsSession {
            worker_id: 0,
            born_at: Instant::now(),
            timings_log: vec![],
            log_file: None,
            curr_stage: None,
            curr_stage_error: false,
            module: None,
        }
    }

    pub fn stage_start(&mut self, stage: CompileStage, stage_extra: Option<StageExtra>) {
        assert!(self.curr_stage.is_none());
        let timestamp = Instant::now().duration_since(self.born_at).as_micros() as u64;
        let stage_extra_r = if let Some(e) = &stage_extra { format!(" ({})", e.render()) } else { String::new() };
        self.write_log(&format!("stage start{stage_extra_r} {:?}", (stage, &self.module)));
        self.curr_stage = Some((stage, stage_extra, self.module.clone(), timestamp));
        self.curr_stage_error = false;
    }

    pub fn stage_end(&mut self, has_error: bool) {
        if let Some((stage, stage_extra, module, start)) = self.curr_stage.take() {
            let timestamp = Instant::now().duration_since(self.born_at).as_micros() as u64;
            let stage_extra_r = if let Some(e) = &stage_extra { format!(" ({})", e.render()) } else { String::new() };
            self.write_log(&format!("stage end{stage_extra_r} {:?}{}", (stage, &module), if self.curr_stage_error { " (has_error)" } else { "" }));
            self.timings_log.push(TimingsEntry {
                stage,
                stage_extra,
                module,
                start,
                end: timestamp,
                has_error: self.curr_stage_error | has_error,
            });
        }
    }

    pub fn write_log(&self, msg: &str) {
        if let Some(file) = &self.log_file {
            let timestamp = Instant::now().duration_since(self.born_at).as_micros() as f64;

            if let Err(e) = write_string(
                file,
                &format!("[Worker-{}][timestamp: {:.2} ms] {msg}\n", self.worker_id, timestamp / 1000.0),
                WriteMode::AppendOrCreate,
            ) {
                eprintln!("Error while writing log (Worker-{}): {e:?}", self.worker_id);
            }
        }
    }
}
