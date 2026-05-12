use super::Redirect;

pub struct Command {
    argv: Vec<String>,
    stdin: Redirect,
    stdout: Redirect
}

// impl Command
impl Command {
    pub fn is_builtin() -> bool {
        true
    }
}