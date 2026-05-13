pub fn run(args: &[String]) -> i32 {
    match args.get(1) {
        None => {
            eprintln!("rsh: expected argument to \"cd\"");
        }
        Some(dir) => {
            if let Err(e) = env::set_current_dir(dir) {
                eprintln!("rsh: {e}");
            }
        }
    }
    1
}