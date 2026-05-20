use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Set;
pub struct Shift;

impl Builtin for Set {
    fn name(&self) -> &'static str { "set" }

    fn run(&self ,args: &[String], shell: &mut Shell) -> i32 {
        if args.is_empty() {
            let mut all: Vec<(&String, &String)> = shell
            .variables
            .iter()
            .chain(shell.env.iter())
            .collect();
            all.sort_by(|a, b| a.0.cmp(b.0));
        for (name, value) in all {
            println!("{}={}", name, value);
        }
        return 0;
        }

        let mut iter = args.iter();
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
                _ => {
                    // Positional parameter assignment is not modeled in the
                    // current Shell struct; we silently accept and ignore the
                    // remaining arguments so that simple scripts still work.
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
    fn name(&self) -> &'static str { "shift" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("shift not implemented");
        0
    }
}