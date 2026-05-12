use crate::lexer::tokenize;
use crate::Token;
use crate::Job;

pub struct Shell {
    jobs: Vec<Job>,
    aliases: HashMap<String, String>,
    env: HashMap<String, String>,
}

impl Shell {
    pub fn run() -> i32 {
        // Todo
        0
    }

    pub fn eval(input: &str) -> i32 {
        let tokens = tokenize(input);   
        // Todo
        0
    } 
}






