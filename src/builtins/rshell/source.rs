use super::Builtin;
use crate::shell::Shell;

pub struct Source;
pub struct Dot;
pub struct Return;

/// Execute the file named by args[0] (or args[1] for trait callers) in the
/// current shell context, line by line. Returns the exit code of the last
/// command, or 1 if the file cannot be read.
pub fn run(args: &[String], shell: &mut Shell) -> i32 {
    let path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("source: usage: source file [args...]");
            return 1;
        }
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("source: {}: {}", path, e);
            return 1;
        }
    };

    let mut last = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        last = shell.eval(trimmed);
    }
    last
}

impl Builtin for Source {
    fn name(&self) -> &'static str {
        "source"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            eprintln!("source: usage: source file [args...]");
            return 1;
        }
        run(&args[1..], shell)
    }
}

impl Builtin for Dot {
    fn name(&self) -> &'static str {
        "."
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            eprintln!(".: usage: . file [args...]");
            return 1;
        }
        run(&args[1..], shell)
    }
}

impl Builtin for Return {
    fn name(&self) -> &'static str {
        "return"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        args.get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(shell.last_status)
    }
}
