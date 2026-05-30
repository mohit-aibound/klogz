use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Read, Write};

use crate::util;

/// Read stdin, write to both stdout and `<log_dir>/<date>/<time>_<label>.log`.
pub fn run(label: Option<String>) -> Result<()> {
    let label = util::sanitize_label(label.as_deref().unwrap_or("capture"));
    let path = util::new_capture_path(&label)?;

    let file = File::create(&path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    let mut sink = BufWriter::new(file);

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    // If stdin is a TTY, treat lines as terminating on newline so feedback is live.
    // Otherwise stream raw bytes for max throughput.
    if atty_stdin() {
        let mut handle = stdin.lock();
        let mut buf = String::new();
        while handle.read_line(&mut buf)? > 0 {
            stdout.write_all(buf.as_bytes())?;
            stdout.flush()?;
            sink.write_all(buf.as_bytes())?;
            buf.clear();
        }
    } else {
        let mut handle = stdin.lock();
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = handle.read(&mut buf)?;
            if n == 0 {
                break;
            }
            stdout.write_all(&buf[..n])?;
            sink.write_all(&buf[..n])?;
        }
        stdout.flush()?;
    }

    sink.flush()?;
    eprintln!("\n[klogz] ✓ saved → {}", path.display());
    Ok(())
}

fn atty_stdin() -> bool {
    // Minimal TTY check without pulling a crate.
    unsafe { libc_isatty(0) }
}

#[cfg(unix)]
unsafe fn libc_isatty(fd: i32) -> bool {
    unsafe extern "C" {
        fn isatty(fd: i32) -> i32;
    }
    unsafe { isatty(fd) == 1 }
}

#[cfg(not(unix))]
unsafe fn libc_isatty(_fd: i32) -> bool {
    false
}
