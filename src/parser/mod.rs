use rsh::lexer::Token;
pub struct Parser {
    tokens: Vec<Token>,
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