use nix::unistd::Pid;
use std::ffi::OsString;
use crate::{Shell, Pipeline, Command, Job, Process, ProcessStatus};

pub fn exec_builtin(shell: &mut Shell, pipeline: Pipeline) -> i32 {
    0
}

pub fn exec_external() {
    
}

fn spawn_pipeline(shell: &mut Shell, pipeline: &Pipeline) -> i32 {
    0
}

fn wait_foreground(pgid: Pid) -> i32 {
    println!("{}", pgid);
    0
}