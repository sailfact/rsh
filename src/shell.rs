use std::collections::HashMap;
use std::env;
use nix::unistd::Pid;

use crate::builtins::rshell::Registry;
use crate::Repl;
use crate::ReadResult; 
use crate::ReplError;
use crate::Job; 
use crate::tokenize;
use crate::Token;
use crate::ProcessStatus;
use crate::executor;
use crate::Parser;

pub struct Shell {
    pub jobs:           Vec<Job>,
    pub aliases:        HashMap<String, String>,
    pub env:            HashMap<String, String>,
    pub registry:       Registry,
    pub last_status:    i32,
}

impl Shell {
        pub fn new() -> Self {
        Shell {
            jobs:           Vec::new(),
            aliases:        HashMap::new(),
            env:            env::vars().collect(),
            registry:       Registry::new(),
            last_status:    0,
        }
    }

    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut repl= Repl::new(String::from("rsh>"))?
            .with_history("~/.rsh_history");
        
        loop {
            self.reap();
            
            match repl.read_line() {
                Ok(ReadResult::Line(input)) => {
                    self.last_status = self.eval(&input);
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

    pub fn eval(&mut self, input: &str) -> i32 {
        let input = input.trim();
        if input.is_empty() {
            return 0;
        }

        let tokens = tokenize(input);

        let pipeline = match Parser::new(tokens).parse() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("rsh: parse error: {e}");
                return 1;
            }
        };

        if pipeline.commands.is_empty() {
            return 0;
        }

        // Single builtin — must run in shell process to mutate Shell state
        if pipeline.commands.len() == 1 && pipeline.commands[0].is_builtin() {
            return dispatch(&pipeline.commands[0], self);
        }

        // Everything else — external commands and pipelines
        executor::execute(self, pipeline)
    }

    pub fn reap(&mut self) {
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
        use nix::unistd::Pid;

        loop {
            match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::Exited(pid, status)) => {
                    self.update_process(pid, ProcessStatus::Exited(status));
                }
                Ok(WaitStatus::Signaled(pid, sig, _)) => {
                    self.update_process(pid, ProcessStatus::Signaled(sig));
                }
                Ok(WaitStatus::Stopped(pid, sig)) => {
                    self.update_process(pid, ProcessStatus::Stopped(sig));
                }
                // No more children to reap
                Ok(WaitStatus::StillAlive) | Err(_) => break,
                _ => continue,
            }
        }

        // Print status of any newly finished background jobs
        self.jobs.retain(|job| {
            if job.is_done() {
                eprintln!("[{}] done\t{}", job.id, job.argv_string());
                false
            } else {
                true
            }
        });
    }
    fn update_process(&mut self, pid: Pid, status: ProcessStatus) {
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






