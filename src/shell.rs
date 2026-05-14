use std::collections::HashMap;
use crate::lexer::tokenize;
use crate::Token;
use crate::Job;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub aliases: HashMap<String, String>,
    pub env: HashMap<String, String>,
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






