use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Set;
pub struct Shift;

impl Builtin for Set {
    fn name(&self) -> &'static str { "set" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("read not implemented");
        0
    }
}

impl Builtin for Shift {
    fn name(&self) -> &'static str { "shift" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("shift not implemented");
        0
    }
}