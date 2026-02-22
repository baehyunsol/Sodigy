use std::io::{Error as IoError, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub enum SubprocessError {
    Timeout,
    IoError(IoError),
}

// It always captures stdout/stderr. If `dump_output` is set, it dumps the
// captured output to the current process' stdout/stderr.
pub fn run(
    binary: &str,
    args: &[&str],
    cwd: &str,
    timeout: f32,  // seconds
    dump_output: bool,
) -> Result<Output, SubprocessError> {
    let timeout = (timeout * 1000.0) as u128;
    let mut child_process = Command::new(binary)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child_process.stdout.take().unwrap();
    let mut child_stderr = child_process.stderr.take().unwrap();

    // VIBE NOTE: I found the test runner deadlocks when the error message is very long.
    //            Gemini 3.1 (via perplexity) told me I should spawn threads that empties
    //            the buffers while the program is running.
    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stdout.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = child_stderr.read_to_end(&mut buf);
        buf
    });

    let started_at = Instant::now();
    let mut sleep_for = 1;

    let status: Result<ExitStatus, SubprocessError> = loop {
        if Instant::now().duration_since(started_at.clone()).as_millis() > timeout {
            child_process.kill()?;
            child_process.wait()?;
            break Err(SubprocessError::Timeout);
        }

        match child_process.try_wait()? {
            Some(status) => {
                break Ok(status);
            },
            None => {
                thread::sleep(Duration::from_millis(sleep_for));
                sleep_for = (sleep_for * 2).min(128);
            },
        }
    };

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();

    if dump_output {
        std::io::stdout().write_all(&stdout)?;
        std::io::stderr().write_all(&stderr)?;
    }

    let status = status?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

impl From<IoError> for SubprocessError {
    fn from(e: IoError) -> SubprocessError {
        SubprocessError::IoError(e)
    }
}
