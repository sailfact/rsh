use crate::shell::Shell;
use super::Builtin;

pub struct Hash;

impl Builtin for Hash {
    fn name(&self) -> &'static str { "hash" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let mut iter = args[1..].iter().peekable();

        if iter.peek().is_none() {
            // print table
            let mut entries: Vec<_> = shell.hash_table.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            for (name, path) in entries {
                println!("{}\t{}", name, path.display());
            }
            return 0;
        }

        let mut status = 0;
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-r" => {
                    shell.hash_table.clear();
                }
                "-d" => {
                    if let Some(name) = iter.next() {
                        shell.hash_table.remove(name.as_str());
                    } else {
                        eprintln!("hash: -d: requires an argument");
                        status = 1;
                    }
                }
                name => {
                    match shell.resolve_command(name) {
                        Some(path) => {
                            shell.hash_table.insert(name.to_string(), path);
                        }
                        None => {
                            eprintln!("hash: {}: not found", name);
                            status = 1;
                        }
                    }
                }
            }
        }
        status
    }
}
