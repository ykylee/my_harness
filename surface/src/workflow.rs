//! Local task files. No grok.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn task_path(home: &Path, id: &str) -> PathBuf {
    home.join(".myharness/handoff/tasks").join(format!("{id}.md"))
}

pub fn task_start(home: &Path, id: &str, title: &str) -> io::Result<PathBuf> {
    let path = task_path(home, id);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let now = chrono_now();
    fs::write(
        &path,
        format!(
            "# {id}\n\n- status: in_progress\n- title: {title}\n- started_at: {now}\n- summary:\n- risks:\n- follow_up:\n"
        ),
    )?;
    Ok(path)
}

pub fn task_end(home: &Path, id: &str, status: &str, summary: &str) -> io::Result<PathBuf> {
    if !valid_status(status) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "task status 는 planned|in_progress|blocked|done",
        ));
    }
    let path = task_path(home, id);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let now = chrono_now();
    let prev = fs::read_to_string(&path).unwrap_or_default();
    let title = prev
        .lines()
        .find(|l| l.starts_with("- title:"))
        .map(|l| l.trim_start_matches("- title:").trim().to_string())
        .unwrap_or_else(|| id.to_string());
    fs::write(
        &path,
        format!(
            "# {id}\n\n- status: {status}\n- title: {title}\n- summary: {summary}\n- ended_at: {now}\n"
        ),
    )?;
    Ok(path)
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

pub fn valid_status(status: &str) -> bool {
    matches!(status, "planned" | "in_progress" | "blocked" | "done")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_then_end() {
        let dir = std::env::temp_dir().join(format!("mh-task-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let p = task_start(&dir, "TASK-1", "hi").unwrap();
        let body = fs::read_to_string(&p).unwrap();
        assert!(body.contains("in_progress"));
        task_end(&dir, "TASK-1", "done", "ok").unwrap();
        let body = fs::read_to_string(&p).unwrap();
        assert!(body.contains("done"));
        let _ = fs::remove_dir_all(&dir);
    }
}
