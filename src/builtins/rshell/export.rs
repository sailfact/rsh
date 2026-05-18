use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Export;

impl Builtin for Export {
    fn name(&self) -> &'static str { "export" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("export not implemented");
        0
    }
}