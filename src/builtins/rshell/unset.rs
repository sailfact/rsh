use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Unset;

impl Builtin for Unset {
    fn name(&self) -> &str { "unset" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("no implemented");
        0
    }
}