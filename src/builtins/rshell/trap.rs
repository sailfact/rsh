use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Trap;

impl Builtin for Trap {
    fn name(&self) -> &str { "trap" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("trap not implemented");
        0
    }
}