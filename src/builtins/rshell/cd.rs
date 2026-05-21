use super::Builtin;
use crate::shell::Shell;

pub struct Cd;

impl Builtin for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        //handle cd ~
        let mut dir = match args.get(1).map(String::as_str) {
            Some("~") | None => shell.variables.get("HOME").cloned().unwrap_or_default(),
            Some(d) if d.starts_with("~/") => {
                let home = std::env::var("HOME").unwrap_or_default();
                format!("{}/{}", home, &d[2..])
            }
            Some(d) => d.to_string(),
        };

        let old = std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().into_owned());
        // handle cd -
        if dir == "-" {
            dir = shell.prev_dir.clone().unwrap_or_default();
            println!("{}", dir);
        }
        shell.prev_dir = old;
        match std::env::set_current_dir(&dir) {
            Ok(_) => {
                if let Some(old_dir) = &shell.prev_dir {
                    shell.env.insert("OLDPWD".into(), old_dir.clone());
                }
                let new = std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or(dir);
                shell.env.insert("PWD".into(), new);
                0
            }
            Err(e) => {
                eprintln!("cd: {}: {}", &dir, e);
                1
            }
        }
    }
}
