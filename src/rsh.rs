use std::io::{
    self, 
    BufRead, 
//    BufReader, 
    Write
};

pub fn r_loop() {
    let args: Vec<&str>;
    let mut delim = [" ", "\t", "\r", "\n", "\x07"]; 
    let status: i32;
    while let Some(line) = prompt("-> ") {
        if line == "exit" {
            break;
        }
        args = split_line(line, delims);
    }
}


pub fn prompt(label: &str) -> Option<String> {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,       // EOF or error
        Ok(_) => Some(line.trim().to_string()),
    }
}


pub fn split_line(line: String, delims: ) -> Vec<String> {

}

/* 
pub fn execute(args: Vec<String>) {
}
*/