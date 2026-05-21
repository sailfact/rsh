// std
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::os::unix::io::RawFd;

// nix
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

use crate::builtins;
use crate::executor;
use crate::jobs::job::Job;
use crate::jobs::process::ProcessStatus;
use crate::lexer::lexer::Lexer;
use crate::lexer::Token;
use crate::parser::parser::Parser;
use crate::repl::{Repl, ReadResult, ReplError};

pub struct Shell {
    pub jobs:        Vec<Job>,
    pub aliases:     HashMap<String, String>,
    pub env:         HashMap<String, String>,   // exported shell variables - inherited by chidren
    pub variables:   HashMap<String, String>,   // unexported shell vars
    pub functions:   HashMap<String, String>,   // shell function bodies
    pub history:     Vec<String>,               // canonical history, read by `history` builtin
    pub last_status: i32,                       // $?
    pub prev_dir:    Option<String>,            // $OLDPWD, used by `cd -`
    pub options:     HashMap<String, bool>,     // set -e, set -x, etc.
    pub hash_table:  HashMap<String, PathBuf>,  // command-peth cache for hash
    pub traps:       HashMap<i32, String>,      // signal -> command
    pub umask:       u32,                       // file-creation mask
    pub shell_pgid:  Pid,                       // process group
    pub tty_fd:      RawFd,                     // controlling terminal fd
}

impl Shell {
    pub fn new() -> Self {
        let shell_pgid = nix::unistd::getpgrp();
        Shell {
            jobs:        Vec::new(),
            aliases:     HashMap::new(),
            env:         env::vars().collect(),
            variables:   HashMap::new(),   
            functions:   HashMap::new(),   
            history:     Vec::new(),
            hash_table:  HashMap::new(),
            last_status: 0,
            prev_dir:    None,
            options:     HashMap::new(),      
            traps:       HashMap::new(),         
            umask:       0o022,
            shell_pgid,
            tty_fd:      nix::libc::STDIN_FILENO,
        }
    }

    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut repl = Repl::new(String::from("rsh> "))?
            .with_history("~/.rsh_history");

        loop {
            self.reap();

            // build prompt
            let cwd = self.env.get("PWD").map(String::as_str).unwrap_or("?");
            let sigil = if self.last_status == 0 { "$" } else { "!" };
            repl.set_prompt(format!("rsh:{} {} ", cwd, sigil));

            match repl.read_line() {
                Ok(ReadResult::Line(input)) => {
                    let trimmed = input.trim().to_string();
                    if !trimmed.is_empty() {
                        // save history to Shell and Repl
                        self.history.push(trimmed.clone());
                        repl.add_history(&trimmed);
                    }
                    self.last_status = self.eval(&trimmed);
                }
                Ok(ReadResult::Eof)         => break,
                Ok(ReadResult::Interrupted) => continue,
                Err(e) => {
                    eprintln!("rsh: {e}");
                    break;
                }
            }
        }

        repl.save_history()?;
        Ok(())
    }

    pub fn eval(&mut self, input: &str) -> i32 {
        let input = input.trim();
        if input.is_empty() {
            return 0;
        }

        let input = self.expand_aliases(input);
        let tokens = Lexer::new(&input).tokenize();

        // Split token stream on semicolons -> one Vec<Token> per Pipeline
        let segments: Vec<Vec<Token>> = tokens
            .split(|t| t == &Token::Semicolon)
            .map(|s| s.to_vec())
            .filter(|s| !s.is_empty()) // ignore tailing ";"
            .collect();

        let mut last = 0;
        for segment in segments {
            last = self.eval_tokens(segment);
        }
        last
    }

    fn expand_aliases(&self, input: &str) -> String {
        let mut words = input.splitn(2, ' ');
        let first = words.next().unwrap_or("");
        if let Some(expansion) = self.aliases.get(first) {
            match words.next() {
                Some(rest) => format!("{} {}", expansion, rest),
                None       => expansion.clone(),
            }
        } else {
            input.to_string()
        }
    }

    fn eval_tokens(&mut self, tokens: Vec<Token>) -> i32 {
        let pipeline = Parser::new(tokens).parse();

        if pipeline.commands.is_empty() {
            return 0;
        }

        // Trivial inline commands
        if pipeline.commands.len() == 1 {
            match pipeline.commands[0].argv[0].as_str() {
                "true"     => return 0,
                "false"    => return 1,
                "break"    => return 128,
                "continue" => return 129,
                _          => {}
            }
        }

        // Single rshell builtin — must run in the shell process
        if pipeline.commands.len() == 1 {
            let name = pipeline.commands[0].argv[0].as_str();
            if builtins::is_rshell_builtin(name) {
                return builtins::exec_shell_builtin(&pipeline.commands[0], self);
            }
        }

        // Everything else — pipelines, uutils, external
        executor::execute(self, pipeline)
    }

    // environment helpers
    pub fn set_env(&mut self, key: &str, value: &str) {
        self.env.insert(key.to_string(), value.to_string());
        unsafe {
            std::env::set_var(key, value);
        }
    }

    pub fn remove_env(&mut self, key: &str) {
        self.env.remove(key);
        unsafe {
            std::env::remove_var(key);
        }
    }

    // command resolution
    pub fn resolve_command(&self, name: &str) -> Option<PathBuf> {
        if let Some(cached) = self.hash_table.get(name) {
            if cached.exists() {
                return Some(cached.clone());
            }
        }
        let path_var = self.env.get("PATH").map(String::as_str).unwrap_or("");
        for dir in env::split_paths(path_var) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    // job helpers
    pub fn next_job_id(&self) -> usize {
        self.jobs.iter().map(|j| j.id).max().unwrap_or(0) + 1
    }

    pub fn find_job_mut(&mut self, spec: Option<&str>) -> Option<&mut Job> {
        match spec {
            None | Some("%+") | Some("%%") => self.jobs.last_mut(),
            Some("%-") => {
                let len = self.jobs.len();
                if len >= 2 {
                    self.jobs.get_mut(len - 2)
                } else {
                    None
                }
            }
            Some(s) => {
                let n: usize = s.strip_prefix('%').unwrap_or(s).parse().ok()?;
                self.jobs.iter_mut().find(|j| j.id == n)
            }
        }
    }

    // process reaping 
    pub fn reap(&mut self) {
        loop {
            match waitpid(
                Pid::from_raw(-1),
                Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED),
            ) {
                Ok(WaitStatus::Exited(pid, code))     => {
                    self.update_process(pid, ProcessStatus::Exited(code));
                }
                Ok(WaitStatus::Signaled(pid, sig, _)) => {
                    self.update_process(pid, ProcessStatus::Signaled(sig));
                }
                Ok(WaitStatus::Stopped(pid, sig))     => {
                    self.update_process(pid, ProcessStatus::Stopped(sig));
                }
                Ok(WaitStatus::StillAlive) | Err(_) => break,
                _ => continue,
            }
        }

        // Print and remove finished background jobs
        self.jobs.retain(|job| {
            if job.is_done() {
                eprintln!("[{}] done\t{}", job.id, job.argv_string());
                false
            } else {
                true
            }
        });
    }

    pub fn update_process(&mut self, pid: Pid, status: ProcessStatus) {
        for job in &mut self.jobs {
            for process in &mut job.processes {
                if process.pid == pid {
                    process.status = status;
                    return;
                }
            }
        }
    }
}