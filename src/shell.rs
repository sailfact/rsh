use std::collections::HashMap;
use std::env;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

use crate::builtins;
use crate::repl::{Repl, ReadResult, ReplError};
use crate::jobs::job::Job;
use crate::jobs::process::ProcessStatus;
use crate::lexer::lexer::Lexer;
use crate::lexer::Token;
use crate::parser::parser::Parser;
use crate::executor;

pub struct Shell {
    pub jobs:           Vec<Job>,
    pub aliases:        HashMap<String, String>,
    pub env:            HashMap<String, String>,
    pub last_status:    i32,
    pub prev_dir:       Option<String>,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            jobs:           Vec::new(),
            aliases:        HashMap::new(),
            env:            env::vars().collect(),
            last_status:    0,
            prev_dir:       None,
        }
    }

    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut repl = Repl::new(String::from("rsh> "))?
            .with_history("~/.rsh_history");

        loop {
            self.reap();

            match repl.read_line() {
                Ok(ReadResult::Line(input)) => {
                    self.last_status = self.eval(&input);
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

    // pub so executor.rs can call it when wait statuses come in
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