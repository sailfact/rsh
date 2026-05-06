use std::io::{self, BufRead, BufReader, Write}

pub fn loop() {
    let line: String;
    let args: Vec<String>;
    let status = true;

    loop {
        println!("-> ");
        io::stdout().flush().unwrap();
        line = rsh_read_line();
        args = rsh_split_line(line);
        status = lsh_execute(args)
        if !status {
            break;
        }
    }

}

pub fn prompt() -> String {

    let reader = BufReader::new(io::stdin());
}

pub fn split_line(line: String) -> Vec<String> {

}

pub fn execute(args: Vec<String) -> i32 {

}