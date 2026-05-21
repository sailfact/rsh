use super::Builtin;
use crate::shell::Shell;

pub struct Exit;

impl Builtin for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let code = args
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(shell.last_status);
        std::process::exit(code);
    }
}
