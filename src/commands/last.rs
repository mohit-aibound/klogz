use anyhow::{Context, Result, bail};
use std::process::Command;

use crate::{config, util};

/// Open the most-recently-modified capture in $EDITOR.
pub fn run() -> Result<()> {
    let files = util::all_captures()?;
    let Some(latest) = files.first() else {
        eprintln!("[klogz] no captures found in {}", config::log_dir()?.display());
        return Ok(());
    };

    println!("Opening: {}", latest.display());
    let editor = config::editor();
    let status = Command::new(&editor).arg(latest).status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }
    Ok(())
}
