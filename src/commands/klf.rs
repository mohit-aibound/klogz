use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::commands::follow;

pub struct Args {
    pub context_keyword: Option<String>,
    pub namespace: String,
    pub prompt_namespace: bool,
    pub tail_lines: u32,
}

/// kubectl pod picker → stream logs + capture (mirrors the zsh `klf` function).
pub fn run(args: Args) -> Result<i32> {
    // 1. Optional context switch.
    if let Some(keyword) = &args.context_keyword {
        switch_context(keyword)?;
    }

    // 2. Optional interactive namespace selection.
    let ns = if args.prompt_namespace {
        match pick_namespace()? {
            Some(n) => n,
            None => return Ok(0),
        }
    } else {
        args.namespace.clone()
    };

    // 3. Pod picker with logs preview.
    let Some(pod) = pick_pod(&ns, args.tail_lines)? else {
        return Ok(0);
    };

    // 4. Label = first two dash-separated segments of the pod name.
    let label: String = pod.split('-').take(2).collect::<Vec<_>>().join("-");

    println!("Streaming logs for: {pod}");
    follow::run(
        label,
        vec![
            "kubectl".into(), "logs".into(), "-f".into(),
            pod, "-n".into(), ns,
        ],
    )
}

fn switch_context(keyword: &str) -> Result<()> {
    let out = Command::new("kubectl")
        .args(["config", "get-contexts", "-o", "name"])
        .output()
        .context("failed to run kubectl config get-contexts")?;
    if !out.status.success() {
        bail!(
            "kubectl config get-contexts failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let needle = keyword.to_lowercase();
    let full = String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|l| l.to_lowercase().contains(&needle))
        .map(|s| s.to_string());

    match full {
        Some(ctx) => {
            println!("Switched context -> {ctx}");
            let status = Command::new("kubectl")
                .args(["config", "use-context", &ctx])
                .stdout(Stdio::null())
                .status()?;
            if !status.success() {
                bail!("kubectl config use-context failed");
            }
            Ok(())
        }
        None => bail!("Context '{keyword}' not found."),
    }
}

fn pick_namespace() -> Result<Option<String>> {
    let kubectl = Command::new("kubectl")
        .args([
            "get", "namespaces",
            "-o", "custom-columns=NAME:.metadata.name",
            "--no-headers",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn kubectl get namespaces")?;

    let fzf = Command::new("fzf")
        .args([
            "--height", "40%",
            "--prompt=Select Namespace > ",
            "--layout=reverse",
        ])
        .stdin(kubectl.stdout.context("kubectl stdout missing")?)
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn fzf")?;

    let out = fzf.wait_with_output()?;
    let sel = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if sel.is_empty() { None } else { Some(sel) })
}

fn pick_pod(ns: &str, tail_lines: u32) -> Result<Option<String>> {
    let kubectl = Command::new("kubectl")
        .args([
            "get", "pods",
            "-n", ns,
            "--sort-by=.metadata.creationTimestamp",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn kubectl get pods")?;

    let preview = format!("kubectl logs --tail={tail_lines} -n {ns} {{1}} 2>&1");
    let less_bind = format!(
        "ctrl-l:execute(kubectl logs --tail={tail_lines} -n {ns} {{1}} | less)"
    );
    let prompt = format!("Select Pod ({ns}) > ");

    let mut fzf = Command::new("fzf")
        .args([
            "--tac",
            "--header-lines=1",
            "--height", "80%",
            "--prompt", &prompt,
            "--layout=reverse",
            "--preview", &preview,
            "--preview-window=right:60%:wrap",
            "--bind", "ctrl-u:preview-half-page-up,ctrl-d:preview-half-page-down",
            "--bind", &less_bind,
        ])
        .stdin(kubectl.stdout.context("kubectl stdout missing")?)
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn fzf")?;

    // Drop our handle so fzf sees EOF promptly when kubectl finishes.
    let _ = fzf.stdin.take();

    let out = fzf.wait_with_output()?;
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if line.is_empty() {
        return Ok(None);
    }
    // First whitespace-separated field is the pod name.
    let pod = line.split_whitespace().next().unwrap_or("").to_string();
    if pod.is_empty() {
        return Ok(None);
    }
    // Also echo the full row so the user sees what they picked.
    let _ = writeln!(std::io::stderr(), "{line}");
    Ok(Some(pod))
}
