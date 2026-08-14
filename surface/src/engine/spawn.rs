//! Headless grok spawn: fds 0/1/2 piped, no PTY, Unix process group.

use std::io;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub struct TurnResult {
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub code: Option<i32>,
}

pub fn spawn_headless(argv: &[String]) -> io::Result<Child> {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn()
}

pub fn kill_group(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
}

/// Run argv, drain pipes, kill the process group on timeout.
pub fn run_turn(argv: &[String], timeout: Duration) -> io::Result<TurnResult> {
    let child = spawn_headless(argv)?;
    let pid = child.id();
    let handle = thread::spawn(move || child.wait_with_output());
    let start = Instant::now();
    loop {
        if handle.is_finished() {
            let out: Output = handle
                .join()
                .map_err(|_| io::Error::other("engine thread panicked"))??;
            return Ok(TurnResult {
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                timed_out: false,
                code: out.status.code(),
            });
        }
        if start.elapsed() >= timeout {
            kill_group(pid);
            let out = handle
                .join()
                .map_err(|_| io::Error::other("engine thread panicked"))??;
            return Ok(TurnResult {
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                timed_out: true,
                code: out.status.code(),
            });
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_kills_sleep() {
        let argv = vec!["sleep".into(), "8".into()];
        let r = run_turn(&argv, Duration::from_millis(200)).unwrap();
        assert!(r.timed_out);
    }

    #[test]
    fn echo_finishes() {
        let argv = vec!["echo".into(), "ok".into()];
        let r = run_turn(&argv, Duration::from_secs(2)).unwrap();
        assert!(!r.timed_out);
        assert!(r.stdout.contains("ok"));
    }
}
