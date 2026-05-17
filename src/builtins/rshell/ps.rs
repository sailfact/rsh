use crate::shell::Shell;
use super::Builtin;

pub struct Ps;

impl Builtin for Ps {
    fn name(&self) -> &str { "ps" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("ps: not implemented");
        0
    }
}