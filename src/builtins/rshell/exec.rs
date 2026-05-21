use super::Builtin;
use crate::shell::Shell;
use nix::unistd::execvp;
use std::ffi::CString;

pub struct Exec;

impl Builtin for Exec {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn run(&self, args: &[String], _shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            return 0;
        }

        let name = match CString::new(args[1].as_str()) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("exec: invalid command name");
                return 1;
            }
        };

        let argv: Vec<CString> = args[1..]
            .iter()
            .filter_map(|s| CString::new(s.as_str()).ok())
            .collect();

        match execvp(&name, &argv) {
            Ok(v) => match v {},
            Err(e) => {
                eprintln!("exec: {}: {}", args[1], e);
                127
            }
        }
    }
}
