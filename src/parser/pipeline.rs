use super::Command;
#[derive(Default)]
pub struct Pipeline {
    commands: Vec<Command>,
    background: bool
}

impl Pipeline {
    pub fn new(commands: Vec<Command>, background: bool) -> Self {
        Self { commands, background }
    }
}