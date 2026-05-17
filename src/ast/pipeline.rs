use super::Command;
#[derive(Default)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool
}

impl Pipeline {
    pub fn new(commands: Vec<Command>, background: bool) -> Self {
        Self { commands, background }
    }
}