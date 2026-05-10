use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use crate::lexer::tokenize;
use crate::jobs;

pub struct Shell {
    jobs: Vec<Job>,
    aliases: HashMap<String, String>,
    env: HashMap<String, String>,
}

impl Shell {
    pub fn run() -> i32 {
        0
    }

    pub fn eval(input: &str) -> i32 {
        let tokens = tokenize(input);   
        0
    } 
}






