use anyhow::{Context, Result};
use std::path::PathBuf;

/// Resolve the root directory where captures are stored.
/// Precedence: $KLOGZ_DIR → $LOGCAP_DIR → ~/logs
pub fn log_dir() -> Result<PathBuf> {
    if let Ok(d) = std::env::var("KLOGZ_DIR") {
        return Ok(PathBuf::from(d));
    }
    if let Ok(d) = std::env::var("LOGCAP_DIR") {
        return Ok(PathBuf::from(d));
    }
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join("logs"))
}

/// Editor used to open captures. $EDITOR → $VISUAL → nvim
pub fn editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nvim".to_string())
}
