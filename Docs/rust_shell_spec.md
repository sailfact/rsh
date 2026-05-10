# Project Spec: Rust Shell

## Overview

A UNIX shell implemented in Rust, built around a classic read-eval-print loop (REPL). The shell supports pipelines, I/O redirection, background execution, job control, aliases, and a small set of built-in commands. It is designed to be correct, safe, and idiomatic Rust — leveraging the ownership system to enforce invariants around process and file-descriptor lifetime.

---

## Goals

- Implement a functional interactive shell suitable for day-to-day use in a terminal
- Handle foreground/background jobs, pipeline setup (fork + exec + dup2), and job control signals (SIGSTOP, SIGCONT, SIGCHLD)
- Provide built-in commands: `cd`, `exit`, `alias`, `jobs`, `fg`, `bg`
- Support I/O redirection (`<`, `>`, `>>`) and pipelines (`|`)
- Maintain a clean separation between parsing, execution, and job management

## Non-Goals

- POSIX compliance (no `$()`, no here-docs, no arithmetic expansion in v1)
- Scripting / non-interactive mode
- Tab completion or advanced readline features

---

## Architecture

The shell is composed of six major subsystems that form a linear pipeline from user input to process execution.

```
User input → Repl → Lexer → Parser → Executor → OS
                                         ↓
                                      Job table
```

### Shell (top-level coordinator)

The root struct that owns all mutable state shared across subsystems.

| Field | Type | Purpose |
|---|---|---|
| `jobs` | `Vec<Job>` | Active and recently completed jobs |
| `aliases` | `HashMap<String, String>` | User-defined command aliases |
| `env` | `HashMap<String, String>` | Shell environment variables |

**Key methods:**

- `run()` — installs signal handlers, then enters the REPL loop
- `eval(input: String)` — drives a single lex → parse → execute cycle; called from the loop

---

### Repl

Handles raw terminal interaction. Responsible for printing the prompt and reading a line of input. Decoupled from `Shell` so the I/O layer can be swapped (e.g., for testing).

- `readline(prompt: &str) -> String` — print prompt, block for input
- `print_output(output: &str)` — write a result line to stdout

---

### Lexer

Converts a raw input string into a flat token stream.

**Token variants:**

| Token | Meaning |
|---|---|
| `Word(String)` | A command name, argument, or filename |
| `Pipe` | `\|` |
| `RedirectIn` | `<` |
| `RedirectOut` | `>` |
| `RedirectAppend` | `>>` |
| `Ampersand` | `&` (background) |
| `Semicolon` | `;` (sequential execution) |

- `tokenize() -> Vec<Token>` — consumes `self.input` and returns the token list

---

### Parser

Consumes the token stream and produces a structured `Pipeline`.

- `parse() -> Pipeline`

**Pipeline:**

```
Pipeline {
    commands: Vec<Command>,
    background: bool,       // trailing & was present
}
```

**Command:**

```
Command {
    argv: Vec<String>,      // argv[0] is the program name
    stdin: Redirect,
    stdout: Redirect,
    is_builtin() -> bool,
}
```

**Redirect variants:**

| Variant | Meaning |
|---|---|
| `Inherit` | Use the shell's own fd (default) |
| `File(String)` | Open a named file |
| `Pipe` | Connected to an adjacent command via a pipe |

---

### Executor

Orchestrates process creation and pipeline wiring. Dispatches to `Builtins` for internal commands.

**Responsibilities:**

1. **`setup_pipes`** — call `pipe(2)` once for each adjacent command pair; annotate `Command.stdin`/`stdout` accordingly
2. **`exec_external`** — `fork(2)`, then in the child: `dup2` the correct pipe ends onto fd 0/1, `setpgid(0, pgid)` to place the child in the job's process group, `close` all other pipe fds, then `execvp`
3. **`execute`** — loop over all commands, build the `Job`, optionally `tcsetpgrp` and `waitpid` for foreground jobs

**Foreground vs. background:**

- Foreground: give the terminal to the job's process group (`tcsetpgrp`), `waitpid(-pgid, WUNTRACED)`, reclaim the terminal on return
- Background: return immediately after pushing the `Job`; reaping happens on the next REPL tick via the `SIGCHLD` handler

---

### Job & Process

Track the runtime state of every process launched by the shell.

```
Job {
    id: usize,
    pgid: Pid,
    processes: Vec<Process>,
    status: JobStatus,      // Running | Stopped | Done
}

Process {
    pid: Pid,
    argv: Vec<String>,
    status: ProcessStatus,  // Running | Exited(i32) | Signaled(Signal)
}
```

- `Job::wait()` — blocking wait on all processes in the group
- `Job::send_signal(sig)` — send a signal to the entire process group via `kill(-pgid, sig)`

---

### Builtins

Pure functions (no OS fork/exec) dispatched by `Executor` when `Command::is_builtin()` returns true.

| Builtin | Behaviour |
|---|---|
| `cd` | Change `$PWD`; update `Shell::env` |
| `exit` | Flush jobs, restore terminal, call `std::process::exit` |
| `alias` | Read or write `Shell::aliases` |
| `jobs` | Print the job table with id, status, and command |
| `fg` | `tcsetpgrp` + `SIGCONT` to the target job's pgid; wait for it |
| `bg` | `SIGCONT` to a stopped job without claiming the terminal |

Builtins are dispatched **before** forking when the pipeline is a single command, so they can mutate `Shell` state directly.

---

## REPL Lifecycle

```
startup:
  install SIGCHLD handler (sets an atomic flag — async-signal-safe)
  Shell::new() → Repl::new()

loop:
  reap_jobs()          ← drain SIGCHLD flag, waitpid(WNOHANG) all tracked pids
  prompt = build_prompt()
  line = Repl::readline(prompt)
  tokens = Lexer::tokenize(line)
  pipeline = Parser::parse(tokens)

  if single builtin (no pipes):
      Builtins::dispatch(cmd, &mut shell)
  else:
      Executor::execute(&mut shell, pipeline)

  store last_status
```

---

## Signal Handling

| Signal | Handler |
|---|---|
| `SIGCHLD` | Sets an `AtomicBool` flag; actual reaping deferred to start of next REPL tick |
| `SIGTTOU` / `SIGTTIN` | Ignored in the shell process to prevent stopping when it reclaims the terminal |
| `SIGINT` / `SIGQUIT` | Ignored in the shell; child process groups inherit the default disposition |

---

## Crate Dependencies (Planned)

| Crate | Purpose |
|---|---|
| `nix` | Safe wrappers around `fork`, `execvp`, `waitpid`, `pipe`, `dup2`, `tcsetpgrp`, `kill` |
| `libc` | Low-level signal constants where `nix` doesn't expose them |
| `rustyline` *(optional)* | Readline-style line editing and history in `Repl` |

---

## Module Layout

```
src/
├── main.rs          # Entry point; constructs Shell and calls run()
├── shell.rs         # Shell struct, eval(), reap_jobs()
├── repl.rs          # Repl struct, readline()
├── lexer.rs         # Lexer, Token
├── parser.rs        # Parser, Pipeline, Command, Redirect
├── executor.rs      # Executor — setup_pipes, fork/exec, waitpid
├── job.rs           # Job, Process, JobStatus, ProcessStatus
└── builtins.rs      # cd, exit, alias, jobs, fg, bg
```

---

## Milestones

| # | Milestone | Definition of Done |
|---|---|---|
| 1 | Single-command execution | `ls`, `echo hello` run correctly |
| 2 | I/O redirection | `<`, `>`, `>>` work on single commands |
| 3 | Pipelines | `ls \| grep foo \| wc -l` works |
| 4 | Background jobs | `sleep 10 &` runs without blocking the shell |
| 5 | Job control | `fg %1`, `bg %1`, Ctrl-Z stops foreground job |
| 6 | Builtins | `cd`, `alias`, `exit`, `jobs` implemented |
| 7 | SIGCHLD reaping | Background job exit is reported on next prompt |
| 8 | Alias expansion | Aliases resolved during lexing/parsing |

---

## Open Questions

- Should alias expansion happen in the `Lexer` or the `Parser`?
- Will `rustyline` be required in v1 or deferred?
- How should the shell report pipeline exit status — last command only (bash default) or bitfield?
- Should `Redirect::Append` be a variant on `Redirect` or encoded as a flag on `Redirect::File`?
