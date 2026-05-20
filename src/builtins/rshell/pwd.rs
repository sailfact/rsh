use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Pwd;

impl Builtin for Pwd {
    fn name(&self) -> &'static str { "pwd" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let physical = args.iter().any(|a| a == "-P");
 
        if !physical {
            if let Some(pwd) = shell.env.get("PWD") {
                if !pwd.is_empty() {
                    println!("{}", pwd);
                    return 0;
                }
            }
        }
 
        match env::current_dir() {
            Ok(p) => {
                let resolved = if physical {
                    p.canonicalize().unwrap_or(p)
                } else {
                    p
                };
                println!("{}", resolved.display());
                0
            }
            Err(e) => {
                eprintln!("pwd: {}", e);
                1
            }
        }
    }
}