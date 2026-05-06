pub fn loop() {
    let line: String;
    let args: Vec<String>;
    let status = true;

    loop {
        println!("-> ");
        line = rsh_read_line();
        args = rsh_split_line(line);
        status = lsh_execute(args)
        if !status {
            break;
        }
    }

}

pub fn read_line() {

}

pub fn split_line(line: String) {

}

pub fn execute(args: Vec<String) {

}