use rsh::Shell;
fn main() {
    if let Err(e) = Shell::new().run() {
        eprintln!("rsh: {e}");
        std::process::exit(1);
    }
}