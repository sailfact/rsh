use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Exec;

impl Builtin for Exec{
    fn name(&self) -> &'static str { "exec" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        // TODO
        println!("exec: not implemented");
        0
    }
}