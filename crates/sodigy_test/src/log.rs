// TODO: use `env_logger` instead
// -> I just found that rustc uses `tracing` instead of `env_logger`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

pub const LOG_IMPORTANT: u16 = 0;
pub const LOG_NORMAL: u16 = 1;
pub const LOG_VERBOSE: u16 = 2;

// TODO: make these configurable
const LOG_THRESHOLD: u16 = 3;

// TODO: I cannot use join function here, I have to make sure that
// it works on all platforms
pub const LOG_FILE_PATH: &str = "./sodigy_compiler_logs.txt";

fn exists(path: &str) -> bool {
    PathBuf::from_str(path).map(|path| path.exists()).unwrap()
}

// I really want to import sodigy_files, but that introduces a cycle...
pub fn sodigy_log(level: u16, mut msg: String) {
    if level < LOG_THRESHOLD {
        if msg.chars().last() != Some('\n') {
            msg = format!("{msg}\n");
        }

        // ugly hack: if mutiple threads call `exists(LOG_FILE_PATH)` at the same time,
        // many of them would get `false`. in order to prevent creating the file multiple
        // times, it runs a meaningless loop
        if !exists(LOG_FILE_PATH) {
            let iter_count = rand::random::<usize>() & 31 | 32;

            for _ in 0..iter_count {
                if exists(LOG_FILE_PATH) {
                    break;
                }
            }
        }

        // if the log file doesn't exist, it creates the file
        if !exists(LOG_FILE_PATH) {
            // there's no way we can handle errors in this crate
            let oo = OpenOptions::new().write(true).create_new(true).to_owned();
            let mut f = oo.open(LOG_FILE_PATH).unwrap();
            f.write_all(msg.as_bytes()).unwrap();
        }

        // if the file does exist, it appends to the file
        else {
            let oo = OpenOptions::new().append(true).to_owned();
            let mut f = oo.open(LOG_FILE_PATH).unwrap();
            f.write_all(msg.as_bytes()).unwrap();
        }
    }
}
