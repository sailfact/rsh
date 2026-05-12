use crate::lexer::Token;
use super::Pipeline;

pub struct Parser {
    tokens: Vec<Token>,
}

// imple Parser
impl Parser{
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
        }
    }
    pub fn parse(&mut self) -> Pipeline {
        let pl = Pipeline::default();
        // To Do Walk token Stream Left to Right
        pl
    }
}