use crate::shell::Shell;
use super::Builtin;

pub struct Unset;

impl Builtin for Unset {
    fn name(&self) -> &'static str { "unset" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            return 0;
        }

        let mut funcs_only = false;
        let mut vars_only  = false;
        let mut names: Vec<&str> = Vec::new();

        let mut iter = args[1..].iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-f" => funcs_only = true,
                "-v" => vars_only  = true,
                _    => names.push(arg.as_str()),
            }
        }

        for name in names {
            if funcs_only {
                shell.functions.remove(name);
            } else if vars_only {
                shell.variables.remove(name);
            } else {
                shell.variables.remove(name);
                shell.remove_env(name);
            }
        }
        0
    }
}
