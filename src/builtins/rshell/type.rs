use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Type;

impl Builtin for Type {
    fn name(&self) -> &str { "type" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("type: no implemented");
        0
    }
}