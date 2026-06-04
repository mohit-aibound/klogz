use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::{config, util};

/// fzf picker over all captures (newest first), then open in $EDITOR.
pub fn run() -> Result<()> {
    let files = util::all_captures()?;
    if files.is_empty() {
        eprintln!("[klogz] no captures found in {}", config::log_dir()?.display());
        return Ok(());
    }

    let mut fzf = Command::new("fzf")
        .args([
            "--height", "80%",
            "--ansi",
            "--preview",
            r#"f="{}"; echo "── $f" && wc -l "$f" && echo "──" && if command -v bat >/dev/null 2>&1; then bat --color=always --style=numbers --line-range :200 "$f"; else tail -60 "$f"; fi"#,
            "--preview-window=right:60%",
            "--prompt=logs > ",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn fzf (is it installed?)")?;

    {
        let stdin = fzf.stdin.as_mut().context("fzf stdin missing")?;
        for f in &files {
            writeln!(stdin, "{}", f.display())?;
        }
    }

    let output = fzf.wait_with_output()?;
    if !output.status.success() {
        // User hit Esc — exit cleanly.
        return Ok(());
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selection.is_empty() {
        return Ok(());
    }

    let editor = config::editor();
    let status = Command::new(&editor).arg(&selection).status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }
    Ok(())
}
