use crate::shell::Shell;
use super::Builtin;

pub struct Ls;  

impl Builtin for Ls {
    fn name(&self) -> &str { "ls" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        println!("ls: not implemented");
        0
    }           
}