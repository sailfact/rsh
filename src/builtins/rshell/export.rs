use super::Builtin;
use crate::shell::Shell;

pub struct Export;

impl Builtin for Export {
    fn name(&self) -> &'static str {
        "export"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() == 1 || (args.len() == 2 && args[1] == "-p") {
            let mut pairs: Vec<(&String, &String)> = shell.env.iter().collect();
            pairs.sort_by_key(|(k, _)| *k);
            for (k, v) in pairs {
                println!("declare -x {}=\"{}\"", k, v);
            }
            return 0;
        }

        let status = 0;
        for arg in &args[1..] {
            if arg == "-p" {
                continue;
            }
            if let Some(eq) = arg.find('=') {
                let key = &arg[..eq];
                let val = &arg[eq + 1..];
                shell.set_env(key, val);
            } else {
                // promote existing shell variable to env, or create empty
                let val = shell.variables.get(arg).cloned().unwrap_or_default();
                shell.set_env(arg, &val);
            }
        }
        status
    }
}
