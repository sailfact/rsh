use crate::shell::Shell;
use super::Builtin;

pub struct Bg;

impl Builtin for Bg {
    fn name(&self) -> &str { "bg" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        // TODO
        println!("bg: not implemented");
        0
    }
}