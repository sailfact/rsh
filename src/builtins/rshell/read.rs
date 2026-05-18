use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Read;

impl Builtin for Read {
    fn name(&self) -> &'static str { "read" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("read not implemented");
        0
    }
}