use crate::lexer::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    File(String),
    Inherit, 
    Pipe,
}

pub struct Parser {
    tokens: Vec<Token>,
}


#[derive(Default)]
pub struct Pipeline {
    commands: Vec<Command>,
    background: bool
}

#[derive(Default)]
pub struct Command {
    argv: Vec<string>,
    stdin: Redirect,
    stdout: Redirect
}

// imple Parser
impl Parser{
    pub fn new(tokens: Vec<Token>) -> self {
        self {
            tokens: tokens,
        }
    }
    pub fn Parse(&mut self) -> Pipeline {
        let mut pl = Pipeline::default();
        // To Do Walk token Stream Left to Right
        pl
    }
}

// impl Pipeline

// impl Command
impl Command {
    pub fn is_builtin() -> bool {
        true
    }
}