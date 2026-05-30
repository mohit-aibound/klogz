use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::util;

/// Spawn `cmd...`, tee its stdout+stderr to a capture file and to our own stdout/stderr.
pub fn run(label: String, cmd: Vec<String>) -> Result<i32> {
    if cmd.is_empty() {
        bail!("no command given; usage: klogz follow <label> -- <cmd> [args...]");
    }

    let label = util::sanitize_label(&label);
    let path = util::new_capture_path(&label)?;
    eprintln!("[klogz] streaming → {}  (Ctrl-C to stop)", path.display());

    let file = File::create(&path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    let sink = Arc::new(Mutex::new(BufWriter::new(file)));

    let mut child = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn: {}", cmd.join(" ")))?;

    let stdout = child.stdout.take().context("missing child stdout")?;
    let stderr = child.stderr.take().context("missing child stderr")?;

    let sink_out = Arc::clone(&sink);
    let sink_err = Arc::clone(&sink);

    let t_out = thread::spawn(move || tee(stdout, std::io::stdout(), sink_out));
    let t_err = thread::spawn(move || tee(stderr, std::io::stderr(), sink_err));

    let status = child.wait()?;
    let _ = t_out.join();
    let _ = t_err.join();

    if let Ok(mut s) = sink.lock() {
        let _ = s.flush();
    }

    Ok(status.code().unwrap_or(if status.success() { 0 } else { 1 }))
}

fn tee<R: Read, W: Write>(mut src: R, mut term: W, sink: Arc<Mutex<BufWriter<File>>>) {
    let mut buf = [0u8; 8 * 1024];
    loop {
        match src.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let _ = term.write_all(&buf[..n]);
                let _ = term.flush();
                if let Ok(mut s) = sink.lock() {
                    let _ = s.write_all(&buf[..n]);
                }
            }
            Err(_) => break,
        }
    }
}
