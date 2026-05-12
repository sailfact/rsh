use super::Command;
#[derive(Default)]
pub struct Pipeline {
    commands: Vec<Command>,
    background: bool
}