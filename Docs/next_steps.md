# Next Steps

A prioritised roadmap for `rsh`, based on an audit of the codebase (2026-07-06)
against the milestones in `rust_shell_spec.md`.

## Where the project stands

Build is clean, `cargo clippy -D warnings` passes, and all 6 unit tests pass.
Measured against the spec's 13 milestones:

| # | Milestone | Status |
|---|---|---|
| 1 | Single-command execution | ✅ Done |
| 2 | I/O redirection (`<`, `>`, `>>`) | ✅ Done |
| 3 | Pipelines | ✅ Done |
| 4 | Background jobs (`&`) | ✅ Done |
| 5 | Job control (`fg`, `bg`, Ctrl-Z) | ✅ Done — signal setup landed in `src/signals.rs` |
| 6 | Builtins | ✅ Done — 23 rshell builtins + uutils coreutils; redirects now honored via fd save/restore |
| 7 | SIGCHLD reaping | ✅ Done — flag handler installed; `reap()` gated on the flag |
| 8 | Alias expansion | ⚠️ First-word only, pre-lex string substitution; fragile (see below) |
| 9 | Non-interactive mode | ✅ Done — `rsh -c`, `rsh script.sh`, piped stdin; exits with last status |
| 10 | Word expansion (`$VAR`, `$?`, `~`, globs, quote semantics) | ✅ Done — `src/expansion.rs`; bare `NAME=value` assignments too |
| 11 | POSIX expansions (`$()`, `<<`, `$((...))`) | ❌ Not started |
| 12 | Scripting constructs (`if`/`while`/`for`, functions) | ❌ Not started (stub fields exist on `Shell`) |
| 13 | Tab completion | ❌ Not started (rustyline defaults only) |

## Defects and gaps found during the audit

1. **No signal handlers are installed.** `CLAUDE.md` and the spec describe
   ignoring `SIGTTOU`/`SIGTTIN` and installing a SIGCHLD flag handler, but
   `Shell::run()` installs nothing — the only signal code is a re-export in
   `src/jobs/mod.rs:7`. Consequences:
   - When `wait_foreground()` reclaims the terminal via `tcsetpgrp`
     (`src/executor.rs:155`) the shell is in a background process group at
     that moment, so the kernel sends it `SIGTTOU` and can stop the shell.
   - `SIGINT`/`SIGQUIT` are not ignored in the shell process, and children
     do not reset dispositions to default before `execvp`.
2. **Panics on syscall failure.** `executor::execute` uses
   `expect()` on `pipe()`/`fork()` (crashes the whole shell), and
   `apply_redirects` uses `expect()` on file open (panics in the forked
   child, printing a Rust backtrace instead of `rsh: foo: No such file`).
3. **Traps are stored but never fired.** The `trap` builtin populates
   `Shell::traps`, but nothing ever executes the registered commands.
4. **Alias expansion is fragile** (`shell.rs::expand_aliases`): it runs once
   on the raw input line before lexing, so only the first word of the first
   command expands — aliases after `;` or `|` are missed, and there's no
   recursive expansion or loop guard.
5. **Parser silently swallows malformed input** — a redirect with no target
   or an empty pipeline segment produces no error, and `&`/`|` in illegal
   positions are ignored (`src/parser/parser.rs:80`).
6. **No `&&` / `||` operators** — the lexer has no tokens for them, and a
   lone `&` mid-line is treated as (ignored) background marker.
7. **Test coverage is thin**: 6 unit tests total (4 lexer, 2 jobs); no
   parser, executor, or end-to-end tests.
8. Leftover TODOs that are actually already resolved: `src/main.rs:3` and
   `src/shell.rs:20,47` (`install_defaults` *is* wired in; `is_login` /
   `is_interactive` *are* detected). Delete the comments.

## Roadmap

Ordered so that each phase unlocks the next; phases 0–2 are small and pay
for everything after.

### Phase 0 — Correctness & hygiene (small, do first) ✅ DONE

- Install signal handling in `Shell::run()` per the spec: ignore
  `SIGTTOU`/`SIGTTIN`/`SIGINT`/`SIGQUIT` in the shell; in each forked child,
  restore default dispositions before `execvp`. Add the SIGCHLD
  `AtomicBool` flag handler so `reap()` only scans when something changed.
- Replace `expect()` in `executor.rs` with graceful errors: parent-side
  `pipe`/`fork` failure prints `rsh: ...` and returns status 1; child-side
  redirect failure prints one line and `exit(1)`; command-not-found exits 127.
- Delete the three stale TODO comments (main.rs, shell.rs).

### Phase 1 — Milestone 9: non-interactive mode ✅ DONE

- Argument handling in `main.rs` / `Shell::run()`:
  `rsh script.sh`, `rsh -c 'cmd'`, and piped stdin (`echo ls | rsh`) all
  evaluate without a prompt or rustyline, exiting with the last command's
  status. The `is_interactive` flag already exists — branch on it.
- This is deliberately before word expansion because it makes the shell
  scriptable by the test suite.

### Phase 2 — Testing foundation ✅ MOSTLY DONE

`tests/cli.rs` (25 end-to-end tests) landed; parser `Result` errors remain.

- Add `tests/` integration suite driving `target/debug/rsh -c '...'`
  (e.g. via `assert_cmd`): cover pipelines, redirects, exit statuses,
  builtins, background jobs. CI already runs `cargo test`.
- Add parser unit tests for the malformed-input cases in gap #5, fixing the
  parser to return `Result` with real error messages as part of it.

### Phase 3 — Milestone 10: word expansion ✅ DONE

Landed as `src/expansion.rs`: the lexer now keeps quote characters in
`Word` tokens and the expansion pass (between parse and dispatch) handles
tilde, `$VAR`/`${VAR}`/`$?`/`$$`/`$0`-`$9`/`$#`, field splitting, globs
(via the `glob` crate, bash-style no-match-stays-literal), quote removal,
and bare `NAME=value` assignments. Alias expansion was left pre-lex for
now. Original design notes below.

The largest functional gap. Suggested design:

- Change `Token::Word(String)` to carry quoting information, e.g.
  `Word(Vec<Segment>)` where `Segment { text, quote: None|Single|Double }` —
  the lexer currently discards quotes (`lexer.rs::read_quoted`), which makes
  correct expansion impossible downstream. Also handle `\` escapes.
- Add an expansion pass between parse and execute, applied per `Command`:
  1. tilde expansion (`~`, `~/x`) on unquoted segments
  2. parameter expansion (`$VAR`, `${VAR}`, `$?`, `$$`, `$0`–`$9`) in
     unquoted and double-quoted segments — sources: `Shell::variables`,
     `Shell::env`, `last_status`
  3. glob expansion (`*`, `?`, `[...]`) on unquoted segments (the `glob`
     crate is the easy path)
  4. field splitting of unquoted expansion results
- This answers the spec's open question: alias expansion should also move
  here (post-tokenize, first word of each command, with a loop guard),
  fixing gap #4.

### Phase 4 — Command lists: `&&`, `||`, proper `;`/`&` handling

- New tokens `AndIf`/`OrIf`; replace the semicolon `split()` in
  `Shell::eval` with a real list parser producing
  `Vec<(Pipeline, Separator)>`. `&` becomes a separator (bash allows
  `a & b`), not just a trailing flag.
- Prerequisite for `if`/`while` in Phase 6.

### Phase 5 — Milestone 11: POSIX expansions

- `$()` command substitution (run pipeline, capture stdout, strip trailing
  newlines) — slots into the Phase 3 expansion pass.
- Here-docs (`<<`, `<<-`) — lexer needs multi-line awareness; REPL needs
  continuation prompts.
- Arithmetic expansion `$((...))`.

### Phase 6 — Milestone 12: scripting constructs

- `if`/`elif`/`else`, `while`/`until`, `for` — requires upgrading the AST
  from flat `Pipeline` to a recursive `CompoundCommand`.
- Functions (populate the existing `Shell::functions`), positional
  parameters + `shift`, `return`, real `break`/`continue` (replacing the
  128/129 sentinel hack in `shell.rs::eval_tokens`).
- Honor `set -e` / `set -x` (the `options` map already exists).
- Fire traps: check `Shell::traps` in the reap/signal path and on `exit`.

### Phase 7 — Milestone 13: tab completion

- Implement a rustyline `Completer`/`Helper` in `repl.rs`: command-name
  completion from builtins + `PATH` (reuse `Shell::resolve_command`'s PATH
  walk), path completion elsewhere on the line.

## Suggested immediate next PR

~~Phase 0 + Phase 1 together~~ — done (signal handling, panic removal,
non-interactive mode, builtin redirects via fd save/restore, and the
`tests/cli.rs` end-to-end suite). Defects 1, 2, and 8 above are fixed;
during that work another was found and fixed: redirects on single rshell
builtins (`echo x > file`) were silently dropped.

Next up: **Phase 3 — word expansion** (with parser error handling from
Phase 2 folded in), which unblocks everything downstream.
