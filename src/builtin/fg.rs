use crate::shell::Shell;
use super::Builtin;

pub struct Fg;

impl Builtin for Fg {
    fn name(&self) -> &str { "fg" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("fg: not implemented");
        0
    }
}