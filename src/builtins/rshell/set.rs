use super::Builtin;
use crate::shell::Shell;

pub struct Set;
pub struct Shift;

impl Builtin for Set {
    fn name(&self) -> &'static str {
        "set"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() <= 1 {
            let mut all: Vec<(&String, &String)> =
                shell.variables.iter().chain(shell.env.iter()).collect();
            all.sort_by(|a, b| a.0.cmp(b.0));
            for (name, value) in all {
                println!("{}={}", name, value);
            }
            return 0;
        }

        let mut iter = args[1..].iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-o" | "+o" => {
                    let enable = arg == "-o";
                    let name = match iter.next() {
                        Some(n) => n,
                        None => {
                            // `set -o` with no arg → print all options
                            print_options(shell);
                            return 0;
                        }
                    };
                    if !is_known_option(name) {
                        eprintln!("set: {}: invalid option name", name);
                        return 2;
                    }
                    shell.options.insert(name.clone(), enable);
                }
                "--" => {
                    // `set -- a b c` replaces the positional parameters,
                    // stored as "1", "2", ... in shell.variables.
                    shell
                        .variables
                        .retain(|k, _| !k.chars().all(|c| c.is_ascii_digit()));
                    for (i, value) in iter.cloned().enumerate() {
                        shell.variables.insert((i + 1).to_string(), value);
                    }
                    break;
                }
                s if s.starts_with('-') || s.starts_with('+') => {
                    let enable = s.starts_with('-');
                    for ch in s.chars().skip(1) {
                        let name = match ch {
                            'e' => "errexit",
                            'x' => "xtrace",
                            'u' => "nounset",
                            'f' => "noglob",
                            'n' => "noexec",
                            'v' => "verbose",
                            _ => {
                                eprintln!("set: -{}: invalid option", ch);
                                return 2;
                            }
                        };
                        shell.options.insert(name.into(), enable);
                    }
                }
                arg => {
                    // A bare word also starts the positional parameters.
                    let mut positionals: Vec<String> = vec![arg.to_string()];
                    positionals.extend(iter.cloned());
                    shell
                        .variables
                        .retain(|k, _| !k.chars().all(|c| c.is_ascii_digit()));
                    for (i, value) in positionals.into_iter().enumerate() {
                        shell.variables.insert((i + 1).to_string(), value);
                    }
                    break;
                }
            }
        }
        0
    }
}

fn print_options(shell: &Shell) {
    for name in &[
        "errexit", "xtrace", "nounset", "noglob", "noexec", "verbose",
    ] {
        let on = shell.options.get(*name).copied().unwrap_or(false);
        println!("{:<10}\t{}", name, if on { "on" } else { "off" });
    }
}

fn is_known_option(name: &str) -> bool {
    matches!(
        name,
        "errexit" | "xtrace" | "nounset" | "noglob" | "noexec" | "verbose"
    )
}

impl Builtin for Shift {
    fn name(&self) -> &'static str {
        "shift"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

        // shift $1..$N positional params stored as "1", "2", ... in variables
        let max = shell
            .variables
            .keys()
            .filter_map(|k| k.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        if n > max {
            eprintln!("shift: shift count out of range");
            return 1;
        }

        for i in 1..=max {
            if i + n <= max {
                let val = shell
                    .variables
                    .get(&(i + n).to_string())
                    .cloned()
                    .unwrap_or_default();
                shell.variables.insert(i.to_string(), val);
            } else {
                shell.variables.remove(&i.to_string());
            }
        }
        0
    }
}
