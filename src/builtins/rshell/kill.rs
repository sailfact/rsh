use crate::shell::Shell;
use super::Builtin;

pub struct Kill;

impl Builtin for Kill {
    fn name(&self) -> &'static str { "kill" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("kill: not implemented");
        0
    }
}