use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Umask;

impl Builtin for Umask {
    fn name(&self) -> &str { "umask" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("umask: not implemented");
        0
    }
}