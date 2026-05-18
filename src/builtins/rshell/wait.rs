use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Wait;

impl Builtin for Wait {
    fn name(&self) -> &str { "wait" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("Wait no implemented");
        0
    }
}