use crate::shell::Shell;
use super::Builtin;

pub struct Mkdir;

impl Builtin for Mkdir {
    fn name(&self) -> &str { "mkdir" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("mkdir: not implemented");
        0
    }
}