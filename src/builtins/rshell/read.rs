use crate::shell::Shell;
use super::Builtin;
use std::io::{self, BufRead, Write};

pub struct Read;

impl Builtin for Read {
    fn name(&self) -> &'static str { "read" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let mut raw     = false;
        let mut prompt  = String::new();
        let mut varnames: Vec<&str> = Vec::new();

        let mut iter = args[1..].iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-r" => raw = true,
                "-p" => {
                    if let Some(p) = iter.next() {
                        prompt = p.clone();
                    }
                }
                _ => varnames.push(arg.as_str()),
            }
        }

        if varnames.is_empty() {
            varnames.push("REPLY");
        }

        if !prompt.is_empty() {
            eprint!("{}", prompt);
            let _ = io::stderr().flush();
        }

        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) => return 1, // EOF
            Err(_) => return 1,
            Ok(_) => {}
        }

        // strip trailing newline
        if line.ends_with('\n') { line.pop(); }
        if line.ends_with('\r') { line.pop(); }

        // backslash-newline continuation (unless -r)
        if !raw {
            line = line.replace("\\n", "\n")
                       .replace("\\t", "\t")
                       .replace("\\\\", "\\");
        }

        // split on IFS (default space/tab)
        let ifs = shell.variables.get("IFS")
            .cloned()
            .unwrap_or_else(|| " \t".to_string());

        let split_fn = |c: char| ifs.contains(c);
        let mut parts: Vec<String> = line.splitn(varnames.len(), split_fn)
            .map(str::to_string)
            .collect();

        // pad with empty strings if fewer parts than vars
        while parts.len() < varnames.len() {
            parts.push(String::new());
        }

        for (var, val) in varnames.iter().zip(parts.iter()) {
            shell.variables.insert(var.to_string(), val.clone());
        }
        0
    }
}
