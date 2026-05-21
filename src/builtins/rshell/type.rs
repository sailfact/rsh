use crate::shell::Shell;
use crate::builtins;
use super::Builtin;

pub struct Type;

impl Builtin for Type {
    fn name(&self) -> &'static str { "type" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            eprintln!("type: usage: type name [name ...]");
            return 1;
        }

        let mut status = 0;
        for name in &args[1..] {
            if shell.aliases.contains_key(name.as_str()) {
                let val = &shell.aliases[name.as_str()];
                println!("{} is aliased to `{}'", name, val);
            } else if shell.functions.contains_key(name.as_str()) {
                println!("{} is a function", name);
            } else if builtins::is_rshell_builtin(name) {
                println!("{} is a shell builtin", name);
            } else if builtins::is_uutils_builtin(name) {
                println!("{} is a shell builtin", name);
            } else if let Some(path) = shell.resolve_command(name) {
                println!("{} is {}", name, path.display());
            } else {
                eprintln!("type: {}: not found", name);
                status = 1;
            }
        }
        status
    }
}
