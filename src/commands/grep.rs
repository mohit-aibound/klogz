use anyhow::{Context, Result};
use std::process::{Command, Stdio};

use crate::config;

/// `rg -i -l <query> <log_dir> | fzf --preview 'rg -i --color=always -C 3 -- $RG_QUERY {}'`.
pub fn run(query: String) -> Result<()> {
    let root = config::log_dir()?;
    if !root.exists() {
        eprintln!("[klogz] no captures found in {}", root.display());
        return Ok(());
    }

    let rg = Command::new("rg")
        .args(["-i", "-l", "--", &query])
        .arg(&root)
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn rg (is ripgrep installed?)")?;

    let preview = "rg -i --color=always -C 3 -- \"$RG_QUERY\" {}";
    let status = Command::new("fzf")
        .env("RG_QUERY", &query)
        .args([
            "--ansi",
            "--preview", preview,
            "--preview-window=right:60%",
        ])
        .stdin(rg.stdout.unwrap_or_else(|| panic!("rg stdout missing")))
        .status()
        .context("failed to spawn fzf (is it installed?)")?;

    // fzf returns non-zero on Esc; treat as clean exit.
    let _ = status;
    Ok(())
}
