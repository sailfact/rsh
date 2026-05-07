use std::io::{self, BufRead, Write};
use std::process::{Command, Child};

use crate::buitins;


const DELIMS: &[char] = &[' ', '\t', '\r', '\n', '\x07'];

pub fn rsh_loop() {
    while let Some(line) = rsh_prompt("-> ") {
        if line == "exit" { break; }
        let args= rsh_split_line(&line);
        let status= rsh_execute(&args);
        println!("{}", status);
    }
}


pub fn rsh_prompt(label: &str) -> Option<String> {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,       // EOF or error
        Ok(_) => Some(line.trim().to_string()),
    }
}


pub fn rsh_split_line(line: &str) -> Vec<String> {
    line.split(DELIMS)
        .filter(|s| !s.is_empty()) 
        .map(|s: &str| s.to_string())
        .collect() 
}

pub fn rsh_launch(args: &[String]) -> i32 {
    if args.is_empty(){ 
        return 1;
    }
    let mut child: Child = Command::new(&args[0])
        .args(&args[1..])
        .spawn()
        .expect("failed to lauch process");
    let status = child.wait().expect("failed to wait process");

    status.code().unwrap_or(1)
}

pub fn rsh_execute(args: Vec<String>) {
    rsh_launch(args);
}
