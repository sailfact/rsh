#[derive(Default)]
pub struct Command {
    argv: Vec<string>,
    stdin: Redirect,
    stdout: Redirect
}

// impl Command
impl Command {
    pub fn is_builtin() -> bool {
        true
    }
}