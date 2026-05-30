# klogz

Structured log capture, browsing, and kubectl log streaming — as a proper CLI tool.

## Install

Requires [Rust](https://rustup.rs) (one-time setup).

```zsh
cargo install --git https://github.com/mohit-aibound/klogz
```

Or clone and install locally:

```zsh
git clone https://github.com/mohit-aibound/klogz
cd klogz
cargo install --path .
```

Verify:

```zsh
klogz --help
```

## Setup

Add to your `~/.zshrc`:

```zsh
export KLOGZ_DIR="$HOME/logs"   # where captures are stored (default: ~/logs)
export EDITOR="nvim"            # used by `look` and `last`

alias lcap='klogz cap'
alias lf='klogz follow'
alias ll='klogz look'
alias lst='klogz last'
alias lg='klogz grep'
alias lc='klogz clean'
alias klf='klogz klf'
```

Then reload: `source ~/.zshrc`

## Commands

### `cap` — capture stdin

Pipes anything into a timestamped log file while still printing to stdout.

```zsh
kubectl logs my-pod | klogz cap auth-crash
grep -i "error" app.log | klogz cap payment-errors
cat big.log | klogz cap                        # label defaults to "capture"
```

Saves to `$KLOGZ_DIR/YYYY-MM-DD/HH-MM-SS_<label>.log`.

---

### `follow` — stream a command and capture it

Runs a command, tees its stdout and stderr to a capture file.

```zsh
klogz follow auth-service -- kubectl logs -f my-pod
klogz follow api-errors -- tail -f /var/log/api.log
```

---

### `look` — browse captures

Opens an [fzf](https://github.com/junegunn/fzf) picker over all captures (newest first) with a live preview. Opens the selected file in `$EDITOR`.

```zsh
klogz look
```

---

### `last` — open the most recent capture

```zsh
klogz last
```

---

### `grep` — search across all captures

Uses [ripgrep](https://github.com/BurntSushi/ripgrep) to find matches, then lets you pick a file in fzf with a highlighted preview.

```zsh
klogz grep "NullPointerException"
klogz grep "connection refused"
```

---

### `clean` — delete old captures

Deletes captures older than N days (default 14), then removes empty date directories.

```zsh
klogz clean          # older than 14 days, asks for confirmation
klogz clean 30       # older than 30 days
klogz clean 7 -y     # older than 7 days, skip confirmation
```

---

### `klf` — kubectl pod picker + stream

Fetches pods in a namespace, lets you pick one in fzf with a live log preview, then streams and captures the logs.

```zsh
klogz klf                       # pods in 'siem' namespace, 100-line preview
klogz klf -n app-classification # different namespace
klogz klf -a                    # interactive namespace picker first
klogz klf -c poc                # switch context to one matching 'poc', then pick pod
klogz klf -t 200                # 200-line log preview in fzf
klogz klf -c staging -n siem    # combine flags
```

**fzf keybindings inside the pod picker:**
- `Ctrl-U` / `Ctrl-D` — scroll preview up/down
- `Ctrl-L` — open full logs in `less`

---

## Runtime dependencies

| Dependency | Required by |
|---|---|
| [fzf](https://github.com/junegunn/fzf) | `look`, `grep`, `klf` |
| [ripgrep](https://github.com/BurntSushi/ripgrep) (`rg`) | `grep` |
| `kubectl` | `klf` |
| `$EDITOR` (or `nvim`) | `look`, `last` |

Install on macOS:

```zsh
brew install fzf ripgrep
```

## Updating

```zsh
cargo install --git https://github.com/mohit-aibound/klogz --force
```
