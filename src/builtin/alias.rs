use crate::shell::Shell;
use super::Builtin;

pub struct Alias;
pub struct Unalias;

impl Builtin for Alias {
    fn name(&self) -> &str { "alias" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() == 1 {
            for (name, value) in &shell.aliases {
                println!("{}='{}'", name, value);
            }
            return 0;
        }

        for arg in &args[1..] {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                eprintln!("alias: invalid format: {}", arg);
                continue;
            }
            let name = parts[0].trim().to_string();
            let value = parts[1].trim_matches('\'').to_string();
            shell.aliases.insert(name, value);
        }
        0
    }
}

impl Builtin for Unalias {
    fn name(&self) -> &str { "unalias" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            eprintln!("unalias: usage: unalias name [name ...]");
            return 1;
        }
        for name in &args[1..] {
            shell.aliases.remove(name);
        }
        0
    }
}