```
rsh/
├── Cargo.toml
└── src/
    ├── main.rs           # thin: instantiate Shell, call shell.run()
    ├── lib.rs            # mod declarations + pub re-exports
    ├── shell.rs          # Shell struct — owns jobs, aliases, env
    ├── repl.rs           # Repl struct — wraps rustyline
    ├── executor.rs       # Executor — execute(), spawn_pipeline(), wait_foreground()
    ├── lexer/
    │   ├── mod.rs        # Lexer struct + tokenize()
    │   └── token.rs      # Token enum
    ├── parser/
    │   ├── mod.rs        # Parser struct + parse()
    │   ├── pipeline.rs   # Pipeline struct
    │   ├── command.rs    # Command struct
    │   └── redirect.rs   # Redirect enum
    ├── jobs/
    │   ├── mod.rs        # Job struct, JobStatus enum
    │   ├── process.rs    # Process struct, ProcessStatus enum
    │   └── signals.rs    # SigchldHandler
    └── builtins/
        ├── mod.rs        # dispatch(), is_builtin()
        ├── cd.rs
        ├── alias.rs
        ├── exit.rs       # exit_b
        └── jobs.rs       # jobs_b (list/fg/bg)
```