mod commands;
mod config;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Structured log capture, browsing, and kubectl log streaming.
#[derive(Parser)]
#[command(name = "klogz", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Tee stdin to stdout and to a timestamped capture file.
    ///
    /// Example: `kubectl logs my-pod | klogz cap auth-crash`
    #[command(alias = "lcap")]
    Cap {
        /// Label appended to the capture filename (default: "capture").
        label: Option<String>,
    },

    /// Run a command, tee its stdout+stderr to a capture file.
    ///
    /// Example: `klogz follow auth-service -- kubectl logs -f my-pod`
    #[command(alias = "lf")]
    Follow {
        /// Label for the capture filename.
        label: String,
        /// The command and its arguments. Use `--` to separate flags.
        #[arg(trailing_var_arg = true, required = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },

    /// Browse all captures with fzf + preview, open in $EDITOR.
    #[command(alias = "ll")]
    Look,

    /// Open the most recent capture in $EDITOR.
    #[command(alias = "lst")]
    Last,

    /// Grep across all captures with ripgrep + fzf.
    #[command(alias = "lg")]
    Grep {
        /// Pattern passed to `rg -i`.
        query: String,
    },

    /// Delete captures older than N days (default 14).
    #[command(alias = "lc")]
    Clean {
        /// Age threshold in days.
        #[arg(default_value_t = 14)]
        days: u64,
        /// Skip confirmation prompt.
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },

    /// kubectl pod picker → stream logs to terminal and capture file.
    Klf {
        /// Switch context to the first one matching this substring.
        #[arg(short = 'c')]
        context: Option<String>,
        /// Namespace to query.
        #[arg(short = 'n', default_value = "siem")]
        namespace: String,
        /// Open an interactive namespace picker first.
        #[arg(short = 'a')]
        ask_namespace: bool,
        /// Number of lines shown in the pod log preview.
        #[arg(short = 't', default_value_t = 100)]
        tail_lines: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let exit = match cli.command {
        Command::Cap { label } => {
            commands::cap::run(label)?;
            0
        }
        Command::Follow { label, cmd } => commands::follow::run(label, cmd)?,
        Command::Look => {
            commands::look::run()?;
            0
        }
        Command::Last => {
            commands::last::run()?;
            0
        }
        Command::Grep { query } => {
            commands::grep::run(query)?;
            0
        }
        Command::Clean { days, yes } => {
            commands::clean::run(days, yes)?;
            0
        }
        Command::Klf {
            context,
            namespace,
            ask_namespace,
            tail_lines,
        } => commands::klf::run(commands::klf::Args {
            context_keyword: context,
            namespace,
            prompt_namespace: ask_namespace,
            tail_lines,
        })?,
    };
    std::process::exit(exit);
}
