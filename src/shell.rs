// std
use std::collections::HashMap;
use std::env;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

// nix
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;

use crate::builtins;
use crate::executor;
use crate::jobs::job::Job;
use crate::jobs::process::ProcessStatus;
use crate::lexer::Token;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::repl::{ReadResult, Repl, ReplError};
use crate::signals;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub aliases: HashMap<String, String>,
    pub env: HashMap<String, String>, // exported shell variables - inherited by chidren
    pub variables: HashMap<String, String>, // unexported shell vars
    pub functions: HashMap<String, String>, // shell function bodies
    pub history: Vec<String>,         // canonical history, read by `history` builtin
    pub last_status: i32,             // $?
    pub prev_dir: Option<String>,     // $OLDPWD, used by `cd -`
    pub options: HashMap<String, bool>, // set -e, set -x, etc.
    pub hash_table: HashMap<String, PathBuf>, // command-peth cache for hash
    pub traps: HashMap<i32, String>,  // signal -> command
    pub umask: u32,                   // file-creation mask
    pub shell_pgid: Pid,              // process group
    pub tty_fd: RawFd,                // controlling terminal fd
    pub is_login: bool,
    pub is_interactive: bool,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        let is_login = std::env::args()
            .next()
            .map(|a| a.starts_with('-'))
            .unwrap_or(false);

        let is_interactive =
            std::env::args().len() == 1 && std::io::IsTerminal::is_terminal(&std::io::stdin());

        let shell_pgid = nix::unistd::getpgrp();
        Shell {
            jobs: Vec::new(),
            aliases: HashMap::new(),
            env: env::vars().collect(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            history: Vec::new(),
            hash_table: HashMap::new(),
            last_status: 0,
            prev_dir: None,
            options: HashMap::new(),
            traps: HashMap::new(),
            umask: 0o022,
            shell_pgid,
            tty_fd: nix::libc::STDIN_FILENO,
            is_login,
            is_interactive,
        }
    }

    /// Run the shell to completion and return its exit status.
    pub fn run(&mut self) -> Result<i32, ReplError> {
        signals::install_shell_handlers(self.is_interactive);

        if self.is_login {
            self.source_file_if_exists("~/.rsh_profile");
        }

        if self.is_interactive {
            self.run_interactive()?;
        } else {
            self.run_noninteractive();
        }
        Ok(self.last_status)
    }

    fn run_interactive(&mut self) -> Result<(), ReplError> {
        self.source_file_if_exists("~/.rshrc");
        let mut repl = Repl::new(String::from("rsh> "))?.with_history("~/.rsh_history");

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
                Ok(ReadResult::Eof) => break,
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

    /// Evaluate input without a prompt: `rsh -c 'cmd'`, `rsh script.sh`,
    /// or a script piped on stdin. Sets `last_status` to the status of the
    /// last command run.
    fn run_noninteractive(&mut self) {
        let args: Vec<String> = env::args().skip(1).collect();

        match args.first().map(String::as_str) {
            Some("-c") => match args.get(1) {
                Some(cmd) => {
                    let cmd = cmd.clone();
                    self.last_status = self.eval(&cmd);
                }
                None => {
                    eprintln!("rsh: -c: option requires an argument");
                    self.last_status = 2;
                }
            },
            Some(flag) if flag.starts_with('-') => {
                eprintln!("rsh: {}: invalid option", flag);
                self.last_status = 2;
            }
            Some(path) => {
                let path = path.to_string();
                match std::fs::read_to_string(&path) {
                    Ok(content) => self.last_status = self.eval(&content),
                    Err(e) => {
                        eprintln!("rsh: {}: {}", path, e);
                        self.last_status = 127;
                    }
                }
            }
            None => {
                // No args and stdin is not a terminal: evaluate stdin.
                use std::io::Read;
                let mut input = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut input) {
                    eprintln!("rsh: stdin: {}", e);
                    self.last_status = 1;
                    return;
                }
                self.last_status = self.eval(&input);
            }
        }
        self.reap();
    }

    /// Evaluate one or more lines of input. Blank lines and `#` comment
    /// lines are skipped; each remaining line is split on `;` into
    /// pipelines. Returns the status of the last command run (or the
    /// current `last_status` if nothing ran).
    pub fn eval(&mut self, input: &str) -> i32 {
        let mut last = self.last_status;

        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let line = self.expand_aliases(line);
            let tokens = Lexer::new(&line).tokenize();

            // Split token stream on semicolons -> one Vec<Token> per Pipeline
            let segments: Vec<Vec<Token>> = tokens
                .split(|t| t == &Token::Semicolon)
                .map(|s| s.to_vec())
                .filter(|s| !s.is_empty()) // ignore trailing ";"
                .collect();

            for segment in segments {
                last = self.eval_tokens(segment);
                self.last_status = last;
            }
        }
        last
    }

    fn expand_aliases(&self, input: &str) -> String {
        let mut words = input.splitn(2, ' ');
        let first = words.next().unwrap_or("");
        if let Some(expansion) = self.aliases.get(first) {
            match words.next() {
                Some(rest) => format!("{} {}", expansion, rest),
                None => expansion.clone(),
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
                "true" => return 0,
                "false" => return 1,
                "break" => return 128,
                "continue" => return 129,
                _ => {}
            }
        }

        // Single rshell builtin — must run in the shell process
        if pipeline.commands.len() == 1 {
            let name = pipeline.commands[0].argv[0].as_str();
            if builtins::is_rshell_builtin(name) {
                return self.run_builtin_with_redirects(&pipeline.commands[0]);
            }
        }

        // Everything else — pipelines, uutils, external
        executor::execute(self, pipeline)
    }

    /// Run an rshell builtin in the shell process, temporarily applying any
    /// file redirections to fd 0/1 and restoring them afterwards (builtins
    /// like `cd`/`export` must not fork, so `> file` is done via dup2 swap).
    fn run_builtin_with_redirects(&mut self, cmd: &crate::ast::Command) -> i32 {
        use crate::ast::Redirect;
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::io::IntoRawFd;

        let redirect_in = matches!(cmd.stdin, Redirect::File(_));
        let redirect_out = matches!(cmd.stdout, Redirect::File(_) | Redirect::Append(_));

        if !redirect_in && !redirect_out {
            return builtins::exec_shell_builtin(cmd, self);
        }

        // Open target files before touching fd 0/1 so failures leave the
        // shell's own descriptors untouched.
        let in_fd = if let Redirect::File(path) = &cmd.stdin {
            match OpenOptions::new().read(true).open(path) {
                Ok(f) => Some(f.into_raw_fd()),
                Err(e) => {
                    eprintln!("rsh: {}: {}", path, e);
                    return 1;
                }
            }
        } else {
            None
        };

        let out_fd = match &cmd.stdout {
            Redirect::File(path) => match OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
            {
                Ok(f) => Some(f.into_raw_fd()),
                Err(e) => {
                    eprintln!("rsh: {}: {}", path, e);
                    if let Some(fd) = in_fd {
                        unsafe { libc::close(fd) };
                    }
                    return 1;
                }
            },
            Redirect::Append(path) => match OpenOptions::new().create(true).append(true).open(path)
            {
                Ok(f) => Some(f.into_raw_fd()),
                Err(e) => {
                    eprintln!("rsh: {}: {}", path, e);
                    if let Some(fd) = in_fd {
                        unsafe { libc::close(fd) };
                    }
                    return 1;
                }
            },
            _ => None,
        };

        // Flush buffered output before swapping fd 1 out from under it.
        let _ = std::io::stdout().flush();

        let saved_in = in_fd.map(|fd| unsafe {
            let saved = libc::dup(0);
            libc::dup2(fd, 0);
            libc::close(fd);
            saved
        });
        let saved_out = out_fd.map(|fd| unsafe {
            let saved = libc::dup(1);
            libc::dup2(fd, 1);
            libc::close(fd);
            saved
        });

        let status = builtins::exec_shell_builtin(cmd, self);

        let _ = std::io::stdout().flush();
        if let Some(saved) = saved_in {
            unsafe {
                libc::dup2(saved, 0);
                libc::close(saved);
            }
        }
        if let Some(saved) = saved_out {
            unsafe {
                libc::dup2(saved, 1);
                libc::close(saved);
            }
        }

        status
    }

    // Environment Helpers
    fn source_file_if_exists(&mut self, path: &str) {
        let resolved = if let Some(rest) = path.strip_prefix("~/") {
            let home = self.env.get("HOME").cloned().unwrap_or_else(|| ".".into());
            format!("{}/{}", home, rest)
        } else {
            path.to_string()
        };

        if std::path::Path::new(&resolved).exists() {
            let args = vec![resolved];
            crate::builtins::rshell::source::run(&args, self);
        }
    }
    pub fn install_defaults(&self) {
        let home = self.env.get("HOME").cloned().unwrap_or_else(|| ".".into());

        let files: &[(&str, &str)] = &[
            (".rsh_profile", include_str!("defaults/rsh_profile")),
            (".rshrc", include_str!("defaults/rshrc")),
            (".rsh_logout", include_str!("defaults/rsh_logout")),
        ];

        for (name, contents) in files {
            let path = format!("{}/{}", home, name);
            if !std::path::Path::new(&path).exists()
                && let Err(e) = std::fs::write(&path, contents)
            {
                eprintln!("rsh: warning: could not create {}: {}", name, e);
            }
        }
    }

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
        if let Some(cached) = self.hash_table.get(name).filter(|c| c.exists()) {
            return Some(cached.clone());
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
        // Only scan when the SIGCHLD handler has flagged a child state change.
        if !signals::take_sigchld() {
            return;
        }
        loop {
            match waitpid(
                Pid::from_raw(-1),
                Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED),
            ) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    self.update_process(pid, ProcessStatus::Exited(code));
                }
                Ok(WaitStatus::Signaled(pid, sig, _)) => {
                    self.update_process(pid, ProcessStatus::Signaled(sig));
                }
                Ok(WaitStatus::Stopped(pid, sig)) => {
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
