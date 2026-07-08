# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rsh` is a UNIX shell implementation in Rust. It provides an interactive REPL with job control, pipelines, I/O redirection, aliases, builtins, and bundled coreutils via the `uutils`/`uu_*` crate family.

## Commands

```bash
cargo build --verbose       # build
cargo run                   # run the shell (interactive REPL)
cargo test --verbose        # run all tests (unit + tests/cli.rs end-to-end)
cargo test <module>         # run tests in a specific module (e.g. cargo test lexer)
cargo clippy -- -D warnings # lint (CI enforces zero warnings)
cargo fmt --check           # check formatting (CI enforces)
cargo fmt                   # fix formatting
```

The shell also runs non-interactively: `rsh -c 'cmd'`, `rsh script.sh`, or a
script piped to stdin — the process exits with the last command's status.
`tests/cli.rs` drives the binary this way (`CARGO_BIN_EXE_rsh`).

CI runs on pushes/PRs to `master` and `dev`.

## Architecture

Input flows linearly through five stages:

```
stdin → Repl → Lexer → Parser → Executor → OS
                                    ↓
                                 Job table
```

**`src/shell.rs`** — Top-level coordinator. Owns all mutable shell state (`jobs`, `aliases`, `env`, `variables`, `functions`, `history`, `last_status`). `run()` installs signal handlers and starts the REPL loop; `eval()` drives lex → parse → execute for each line.

**`src/repl.rs`** — Terminal interaction via `rustyline`. Provides `readline()` with prompt, and history load/save.

**`src/lexer/`** — Tokenizes a raw input string into a flat `Vec<Token>`. Tokens: `Word`, `Pipe`, `RedirectIn`, `RedirectOut`, `RedirectAppend`, `Ampersand`, `Semicolon`. Quoted and unquoted runs of the same word are glued into a single `Word` token.

**`src/parser/`** — Consumes tokens and emits a `Pipeline { commands: Vec<Command>, background: bool }`. Each `Command` holds `argv`, a stdin `Redirect`, and a stdout `Redirect` (variants: `Inherit`, `File(String)`, `Pipe`).

**`src/expansion.rs`** — Word expansion pass between parse and dispatch: tilde, `$VAR`/`${VAR}`/`$?`/`$$`/`$0`-`$9`/`$#`, field splitting, globbing, and quote removal (the lexer keeps quote chars in `Word` tokens). Redirect targets expand without splitting. Bare `NAME=value` assignments are handled in `Shell::try_assignment` before expansion.

**`src/executor.rs`** — Forks and execs each command in a pipeline, wires stdin/stdout with `dup2`, and assigns process groups (`setpgid`). Uses `nix` for all syscall wrappers.

**`src/jobs/`** — Tracks live process state. `Job { id, pgid, processes, status }` / `Process { pid, argv, status }`. Provides `wait()` and `send_signal()`.

**`src/builtins/`** — Two sub-namespaces:
- `rshell/` — 23 shell builtins (`cd`, `exit`, `alias`, `export`, `fg`, `bg`, `jobs`, `wait`, `trap`, `kill`, `read`, `set`, `source`, `type`, `history`, `echo`, `pwd`, `umask`, `hash`, `unset`, …)
- `coreutils/` — Thin wrappers around `uu_*` crates grouped into `file_util`, `text_util`, `system_util`, `hash_util`, `misc_util`

**`src/ast/`** — Plain data types: `Command`, `Pipeline`, `Redirect`.

## Signal Handling & Job Control

Implemented in `src/signals.rs`; `Shell::run()` installs handlers before any fork.

- **SIGCHLD**: Sets an atomic flag; jobs are reaped at the start of the next REPL tick (not inside the handler). `Shell::reap()` returns early when the flag is unset.
- **SIGTTOU/SIGTTIN/SIGINT/SIGQUIT/SIGTSTP**: Ignored in interactive shells so the shell itself is never stopped or killed; forked children restore default dispositions before exec.
- Job control (`setpgid` + `tcsetpgrp`) only applies to interactive shells; non-interactive children stay in the shell's process group, like `sh -c`.
- Foreground jobs: `tcsetpgrp` hands the terminal to the job; wait with `WUNTRACED`. Pipeline exit status is the last command's status.
- Background jobs: pushed to the job table; reaped on next SIGCHLD tick.
- Redirects on single rshell builtins are applied in-process via fd save/`dup2`/restore (`Shell::run_builtin_with_redirects`) since builtins like `cd` must not fork.

## Key Reference Docs

- `Docs/rust_shell_spec.md` — Full architecture spec, goals, non-goals, and milestone plan.
- `Docs/Commands.md` — Builtin reference with priority levels.
