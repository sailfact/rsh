use super::Builtin;
use crate::shell::Shell;

pub struct History;

impl Builtin for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.get(1).map(String::as_str) == Some("-c") {
            shell.history.clear();
            return 0;
        }

        let limit: usize = args
            .get(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(shell.history.len());

        let total = shell.history.len();
        let start = total.saturating_sub(limit);

        for (i, cmd) in shell.history[start..].iter().enumerate() {
            println!("{:5}  {}", start + i + 1, cmd);
        }
        0
    }
}
