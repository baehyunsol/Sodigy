use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

pub const LOG_IMPORTANT: u16 = 0;
pub const LOG_NORMAL: u16 = 1;
pub const LOG_VERBOSE: u16 = 2;

// TODO: make these configurable
const LOG_THRESHOLD: u16 = 3;
pub const LOG_FILE_PATH: &str = "./sodigy_compiler_logs.txt";

// I really want to import sodigy_files, but that introduces a cycle...
pub fn sodigy_log(level: u16, mut msg: String) {
    if msg.chars().last() != Some('\n') {
        msg = format!("{msg}\n");
    }

    // TODO: if multiple threads call `sodigy_log` at once, `path.exists()` does not work
    if level < LOG_THRESHOLD {
        // if the log file doesn't exist, it creates the file
        if !PathBuf::from_str(LOG_FILE_PATH).map(
            |path| path.exists()
        ).unwrap() {
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
