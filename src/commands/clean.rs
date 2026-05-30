use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::config;

/// Delete `*.log` captures older than `days` days, then prune empty date dirs.
pub fn run(days: u64, yes: bool) -> Result<()> {
    let root = config::log_dir()?;
    if !root.exists() {
        eprintln!("[klogz] no captures found in {}", root.display());
        return Ok(());
    }

    let cutoff = SystemTime::now() - Duration::from_secs(days * 86_400);
    let mut victims = Vec::new();
    collect_old(&root, cutoff, &mut victims)?;

    println!("Found {} file(s) older than {} days.", victims.len(), days);
    if victims.is_empty() {
        return Ok(());
    }

    if !yes {
        print!("Delete them? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !matches!(line.trim(), "y" | "Y" | "yes") {
            return Ok(());
        }
    }

    for f in &victims {
        if let Err(e) = std::fs::remove_file(f) {
            eprintln!("[klogz] failed to remove {}: {e}", f.display());
        }
    }
    prune_empty_dirs(&root)?;
    println!("Done.");
    Ok(())
}

fn collect_old(
    dir: &Path,
    cutoff: SystemTime,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir failed: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_old(&path, cutoff, out)?;
        } else if ft.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("log")
        {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if mtime < cutoff {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Bottom-up walk; remove any directory left empty (skip the root itself).
fn prune_empty_dirs(root: &Path) -> Result<()> {
    let mut dirs = Vec::new();
    collect_dirs(root, &mut dirs)?;
    // Deepest first.
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for d in dirs {
        if d == root {
            continue;
        }
        if std::fs::read_dir(&d)?.next().is_none() {
            let _ = std::fs::remove_dir(&d);
        }
    }
    Ok(())
}

fn collect_dirs(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    out.push(dir.to_path_buf());
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_dirs(&entry.path(), out)?;
        }
    }
    Ok(())
}
