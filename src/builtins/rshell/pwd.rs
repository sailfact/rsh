use crate::shell::Shell;
use super::Builtin;

pub struct Pwd;

impl Builtin for Pwd {
    fn name(&self) -> &str { "pwd" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("pwd: not implemented");
        0
    }
}