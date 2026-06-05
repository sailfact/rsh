# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rsh` is a UNIX shell implementation in Rust. It provides an interactive REPL with job control, pipelines, I/O redirection, aliases, builtins, and bundled coreutils via the `uutils`/`uu_*` crate family.

## Commands

```bash
cargo build --verbose       # build
cargo run                   # run the shell
cargo test --verbose        # run all tests
cargo test <module>         # run tests in a specific module (e.g. cargo test lexer)
cargo clippy -- -D warnings # lint (CI enforces zero warnings)
cargo fmt --check           # check formatting (CI enforces)
cargo fmt                   # fix formatting
```

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

**`src/executor.rs`** — Forks and execs each command in a pipeline, wires stdin/stdout with `dup2`, and assigns process groups (`setpgid`). Uses `nix` for all syscall wrappers.

**`src/jobs/`** — Tracks live process state. `Job { id, pgid, processes, status }` / `Process { pid, argv, status }`. Provides `wait()` and `send_signal()`.

**`src/builtins/`** — Two sub-namespaces:
- `rshell/` — 23 shell builtins (`cd`, `exit`, `alias`, `export`, `fg`, `bg`, `jobs`, `wait`, `trap`, `kill`, `read`, `set`, `source`, `type`, `history`, `echo`, `pwd`, `umask`, `hash`, `unset`, …)
- `coreutils/` — Thin wrappers around `uu_*` crates grouped into `file_util`, `text_util`, `system_util`, `hash_util`, `misc_util`

**`src/ast/`** — Plain data types: `Command`, `Pipeline`, `Redirect`.

## Signal Handling & Job Control

- **SIGCHLD**: Sets an atomic flag; jobs are reaped at the start of the next REPL tick (not inside the handler).
- **SIGTTOU/SIGTTIN**: Ignored so the shell process itself is never stopped.
- Foreground jobs: `tcsetpgrp` hands the terminal to the job; wait with `WUNTRACED`.
- Background jobs: pushed to the job table; reaped on next SIGCHLD tick.

## Key Reference Docs

- `Docs/rust_shell_spec.md` — Full architecture spec, goals, non-goals, and milestone plan.
- `Docs/Commands.md` — Builtin reference with priority levels.
