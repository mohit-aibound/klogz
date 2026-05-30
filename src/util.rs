use anyhow::{Context, Result};
use chrono::Local;
use std::path::PathBuf;

use crate::config;

/// Sanitize a label: spaces→dashes, strip anything non [A-Za-z0-9_-].
pub fn sanitize_label(label: &str) -> String {
    let dashed = label.replace(' ', "-");
    let cleaned: String = dashed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "capture".to_string()
    } else {
        cleaned
    }
}

/// Build the destination file path for a new capture and ensure the date dir exists.
/// Returns `<log_dir>/YYYY-MM-DD/HH-MM-SS_<label>.log`.
pub fn new_capture_path(label: &str) -> Result<PathBuf> {
    let root = config::log_dir()?;
    let now = Local::now();
    let day = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H-%M-%S").to_string();

    let dir = root.join(&day);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create capture dir: {}", dir.display()))?;

    let filename = format!("{time}_{label}.log");
    Ok(dir.join(filename))
}

/// Walk the log directory and return all `*.log` files, newest-first by mtime.
pub fn all_captures() -> Result<Vec<PathBuf>> {
    let root = config::log_dir()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut out: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    walk(&root, &mut out)?;
    out.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(out.into_iter().map(|(_, p)| p).collect())
}

fn walk(
    dir: &std::path::Path,
    acc: &mut Vec<(std::time::SystemTime, PathBuf)>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir failed: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            walk(&path, acc)?;
        } else if ft.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("log")
        {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            acc.push((mtime, path));
        }
    }
    Ok(())
}
