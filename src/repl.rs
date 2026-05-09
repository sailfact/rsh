pub fn readline(label: &str) -> Option<String> {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,       // EOF or error
        Ok(_) => Some(line.trim().to_string()),
    }
}

pub fn print_output(output: &str){
    println!(&str);
}