use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub struct Repl{
    editor: rustyline::Editor
}
pub fn read_line(label: &str) -> Option<String> {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,       // EOF or error
        Ok(_) => Some(line.trim().to_string()),
    }
    line
}

pub fn print_output(output: &str){
    println!("{:#?}",&str);
}