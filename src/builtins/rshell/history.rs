use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct History;

impl Builtin for History {
    fn name(&self) -> &str { "history" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("history not implemented");
        0
    }
}