use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Jobs;

impl Builtin for Jobs {
    fn name(&self) -> &str { "jobs" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("jobs not implemented");
        0
    }
}