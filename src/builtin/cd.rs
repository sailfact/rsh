use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Cd;

impl Builtin for Cd {
    fn name(&self) -> &str { "cd" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let target = match args.get(1) {
            Some(path) => path.clone(),
            None => match env::var("HOME") {
                Ok(home) => home,
                Err(_) => {
                    eprintln!("cd: HOME not set");
                    return 1;
                }
            }
        };

        match env::set_current_dir(&target) {
            Ok(_) => {
                shell.env.insert("PWD".into(), target);
                0
            }
            Err(e) => {
                eprintln!("cd: {}: {}", target, e);
                1
            }
        }
    }
}