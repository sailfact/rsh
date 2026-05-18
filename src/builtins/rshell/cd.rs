use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Cd;

impl Builtin for Cd {
    fn name(&self) -> &'static str { "cd" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let dir = args.get(0).map(String::as_str).unwrap_or("~");
        match std::env::set_current_dir(dir) {
            Ok(_)  => 0,
            Err(e) => { eprintln!("cd: {}: {}", dir, e); 1 }
        }
    }
}