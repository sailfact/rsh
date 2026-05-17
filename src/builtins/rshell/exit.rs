use crate::shell::Shell;
use super::Builtin;

pub struct Exit;

impl Builtin for Exit {
    fn name(&self) -> &str { "exit" }

    fn run(&self, _args: &[String], _shell: &mut Shell) -> i32 {
        std::process::exit(0);
    }
}