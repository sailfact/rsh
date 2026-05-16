use crate::shell::Shell;
use super::Builtin;

pub struct Rm;

impl Builtin for Rm {
    fn name(&self) -> &str { "rm" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("rm: not implemented");
        0
    }
}