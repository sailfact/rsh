use crate::shell::Shell;
use super::Builtin;

pub struct Echo;

impl Builtin for Echo {
    fn name(&self) -> &'static str { "echo" }

    fn run(&self, args: &[String], _shell: &mut Shell) -> i32 {
        let mut newline  = true;
        let mut escapes  = false;
        let mut word_start = 1;

        // parse flags: -n, -e, -E (and combinations like -ne)
        for arg in &args[1..] {
            if arg.starts_with('-') && arg.len() > 1 && arg[1..].chars().all(|c| matches!(c, 'n' | 'e' | 'E')) {
                for c in arg[1..].chars() {
                    match c {
                        'n' => newline = false,
                        'e' => escapes = true,
                        'E' => escapes = false,
                        _   => {}
                    }
                }
                word_start += 1;
            } else {
                break;
            }
        }

        let words = &args[word_start..];
        let output = words.join(" ");

        let output = if escapes {
            interpret_escapes(&output)
        } else {
            output
        };

        if newline {
            println!("{}", output);
        } else {
            print!("{}", output);
        }
        0
    }
}

fn interpret_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n')  => out.push('\n'),
            Some('t')  => out.push('\t'),
            Some('r')  => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('0')  => out.push('\0'),
            Some('a')  => out.push('\x07'),
            Some('b')  => out.push('\x08'),
            Some('e')  => out.push('\x1B'),
            Some('f')  => out.push('\x0C'),
            Some('v')  => out.push('\x0B'),
            Some(other) => { out.push('\\'); out.push(other); }
            None        => out.push('\\'),
        }
    }
    out
}
