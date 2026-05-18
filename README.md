# rsh
```
src
├── src/ast
│   ├── src/ast/command.rs
│   ├── src/ast/mod.rs
│   ├── src/ast/pipeline.rs
│   └── src/ast/redirect.rs
├── src/builtins
│   ├── src/builtins/coreutils
│   │   ├── src/builtins/coreutils/file_util.rs
│   │   ├── src/builtins/coreutils/hash_util.rs
│   │   ├── src/builtins/coreutils/misc_util.rs
│   │   ├── src/builtins/coreutils/mod.rs
│   │   ├── src/builtins/coreutils/system_util.rs
│   │   └── src/builtins/coreutils/text_util.rs
│   ├── src/builtins/mod.rs
│   └── src/builtins/rshell
│       ├── src/builtins/rshell/alias.rs
│       ├── src/builtins/rshell/bg.rs
│       ├── src/builtins/rshell/cd.rs
│       ├── src/builtins/rshell/exec.rs
│       ├── src/builtins/rshell/exit.rs
│       ├── src/builtins/rshell/export.rs
│       ├── src/builtins/rshell/fg.rs
│       ├── src/builtins/rshell/hash.rs
│       ├── src/builtins/rshell/history.rs
│       ├── src/builtins/rshell/jobs.rs
│       ├── src/builtins/rshell/kill.rs
│       ├── src/builtins/rshell/mod.rs
│       ├── src/builtins/rshell/ps.rs
│       ├── src/builtins/rshell/pwd.rs
│       ├── src/builtins/rshell/read.rs
│       ├── src/builtins/rshell/set.rs
│       ├── src/builtins/rshell/source.rs
│       ├── src/builtins/rshell/trap.rs
│       ├── src/builtins/rshell/type.rs
│       ├── src/builtins/rshell/umask.rs
│       ├── src/builtins/rshell/unset.rs
│       └── src/builtins/rshell/wait.rs
├── src/executor.rs
├── src/external.rs
├── src/jobs
│   ├── src/jobs/job.rs
│   ├── src/jobs/mod.rs
│   └── src/jobs/process.rs
├── src/lexer
│   ├── src/lexer/lexer.rs
│   ├── src/lexer/mod.rs
│   └── src/lexer/token.rs
├── src/lib.rs
├── src/main.rs
├── src/parser
│   ├── src/parser/mod.rs
│   └── src/parser/parser.rs
├── src/repl.rs
└── src/shell.rs
```


## Priority Order
| # | Priority | Commands |
|---|----------|----------|
|**1**| Must-have shell builtins|cd, exit, alias, export, unset, source, jobs, fg, bg, wait, trap, read, type, pwd|
|**2**| High-value uutils| ls, cat, echo, cp, mv, rm, mkdir, grep, head, tail, wc, sort|
|**3**| Scripting support| test, [, expr, true, false, sleep, printf|
|**4**| Nice to have|tee, tac, cut, tr, env, timeout, xargs