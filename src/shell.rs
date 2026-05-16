use std::collections::HashMap;
use std::f32::consts::E;
use crate::ReadResult;
use crate::ReplError;
use crate::lexer;
use crate::Token;
use crate::Job;
use crate::Repl;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub aliases: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub last_status: i32,
}

impl Shell {
        pub fn new() -> Self {
        Shell {
            jobs: Vec::new(),
            aliases: HashMap::new(),
            env: std::env::vars().collect(),
            last_status: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut repl= Repl::new(String::from("rsh>"))?
            .with_history("~/.rsh_history");
        
        loop {
            self.reap_jobs();
            
            match repl.read_line() {
                Ok(ReadResult::Line(input)) => {
                    self.last_status = self.eval(&input);
                }
                Ok(ReadResult::Eof) => break,
                Ok(ReadResult::Interrupted) => continue,
                Err(e) => {
                    eprintln!("rsh: {e}"); 
                    break; 
                } 
            }
        }
        repl.save_history()?;
        Ok(())
    }

    pub fn eval(&mut self, input: &str) -> i32 {
        let tokens = lexer::tokenize(input);   
        // Todo
        0
    } 

    pub fn reap_jobs(&mut self) {

    }
}






