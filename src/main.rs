use rsh::Shell;

// TODO Wire install_defaults into main.rs:
fn main() {
    if let Err(e) = Shell::new().run() {
        eprintln!("rsh: {e}");
        std::process::exit(1);
    }
}