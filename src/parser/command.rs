use super::Redirect;

pub struct Command {
    pub argv: Vec<String>,
    pub stdin: Redirect,
    pub stdout: Redirect
}

// impl Command
impl Command {
    pub fn new(argv: Vec<String>, stdin: Redirect, stdout:Redirect) -> Self {
        Self { argv, stdin, stdout }
    }
    pub fn is_builtin() -> bool {
        true
    }
}