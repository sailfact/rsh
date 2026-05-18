// src/builtins/rshell/mod.rs
pub mod cd;
pub mod exit;
pub mod alias;
pub mod export;
pub mod source;
pub mod set;
pub mod unset;
pub mod fg;
pub mod bg;
pub mod jobs;
pub mod pwd;
pub mod kill;
pub mod wait;
pub mod trap;
pub mod umask;
pub mod read;
pub mod r#type;
pub mod hash;
pub mod history;



use std::collections::HashMap;
use crate::ast::Command;
use crate::shell::Shell;

pub trait Builtin {
    fn name(&self) -> &str;
    fn run(&self, args: &[String], shell: &mut Shell) -> i32;
}

pub struct Registry {
    builtins: HashMap<&'static str, Box<dyn Builtin>>,
}

impl Registry {
    pub fn new() -> Self {
        let mut r = Self { builtins: HashMap::new() };
        r.register(cd::Cd);
        r.register(exit::Exit);
        r.register(alias::Alias);
        r.register(export::Export);
        r.register(unset::Unset);
        r.register(fg::Fg);
        r.register(bg::Bg);
        r.register(jobs::Jobs);
        r.register(pwd::Pwd);
        r.register(wait::Wait);
        r.register(kill::Kill);
        r.register(trap::Trap);
        r.register(umask::Umask);
        r.register(read::Read);
        r.register(source::Source);
        r.register(r#type::Type);
        r.register(hash::Hash);
        r.register(history::History);
        // adding a new builtin = one line here, nothing else changes
        r
    }

    fn register(&mut self, b: impl Builtin + 'static) {
        self.builtins.insert(b.name(), Box::new(b));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Builtin> {
        self.builtins.get(name).map(|b| b.as_ref())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }
}

pub fn dispatch(cmd: &Command, shell: &mut Shell) -> i32 {
    match cmd.argv[0].as_str() {
        // Directory
        "cd"           => Cd::run(&cmd.argv[1..], shell),
        // Session
        "exit"         => Exit::run(&cmd.argv[1..]),
        "exec"         => Exec::run(&cmd.argv[1..], shell),
        "alias"        => Alias::run(&cmd.argv[1..], shell),

        // Removes an alias from shell.aliases
        "unalias"      => Alias::unalias(&cmd.argv[1..], shell),

        // Adds/updates shell.env, marking vars for export to children
        "export"       => export::run(&cmd.argv[1..], shell),

        // Removes a variable from shell.env
        "unset"        => unset::run(&cmd.argv[1..], shell),

        // Sets shell options or positional parameters
        "set"          => set::run(&cmd.argv[1..], shell),

        // Shifts positional parameters left by N (default 1)
        "shift"        => set::shift(&cmd.argv[1..], shell),

        // ── Scripting control ─────────────────────────────────────────────
        // Runs a script file in the current shell context (no fork)
        "source" | "." => source::run(&cmd.argv[1..], shell),

        // Returns from a sourced file or shell function
        "return"       => source::do_return(&cmd.argv[1..], shell),

        // Loop control — actual flow handled by the scripting layer,
        // dispatch just needs to return a recognisable sentinel status
        "break"        => 128,  // scripting layer checks for this
        "continue"     => 129,  //

        // ── Job control ───────────────────────────────────────────────────
        // Lists all jobs in shell.jobs
        "jobs"         => jobs::run(&cmd.argv[1..], shell),

        // Sends SIGCONT to job, calls tcsetpgrp, waits in foreground
        "fg"           => fg::run(&cmd.argv[1..], shell),

        // Sends SIGCONT to job, leaves running in background
        "bg"           => bg::run(&cmd.argv[1..], shell),

        // Waits for a specific job or pid to finish
        "wait"         => wait::run(&cmd.argv[1..], shell),

        // Sends a signal to a job/pid tracked in shell.jobs
        "kill"         => kill::run(&cmd.argv[1..], shell),

        // ── Signal & process control ──────────────────────────────────────
        // Registers a command to run when the shell receives a signal
        "trap"         => trap::run(&cmd.argv[1..], shell),

        // ── Filesystem ────────────────────────────────────────────────────
        // Prints current working directory via std::env::current_dir()
        "pwd"          => pwd::run(&cmd.argv[1..]),

        // Gets/sets the file creation mask
        "umask"        => umask::run(&cmd.argv[1..], shell),

        // ── I/O ───────────────────────────────────────────────────────────
        // Reads a line from stdin into a shell variable
        "read"         => read::run(&cmd.argv[1..], shell),

        // Echo — inline here since it's a one-liner;
        // swap for uu_echo in coreutils dispatch if you want flag support
        "echo"         => { println!("{}", cmd.argv[1..].join(" ")); 0 }

        // ── Trivial status ────────────────────────────────────────────────
        "true"         => 0,
        "false"        => 1,

        // ── Introspection ─────────────────────────────────────────────────
        // Resolves whether a name is a builtin, alias, function, or external
        "type"         => r#type::run(&cmd.argv[1..], shell),

        // Caches PATH lookups to speed up repeated external command resolution
        "hash"         => hash::run(&cmd.argv[1..], shell),

        // Reads/writes readline history
        "history"      => history::run(&cmd.argv[1..], shell),

        // ── Process info ──────────────────────────────────────────────────
        "ps"           => ps::run(&cmd.argv[1..]),

        _ => {
            eprintln!("shell: unknown builtin: {}", cmd.argv[0]);
            1
        }
    }
}