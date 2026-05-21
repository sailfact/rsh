use rsh::Shell;

// TODO Wire install_defaults into main.rs:
fn main() {
    let mut shell = Shell::new();
    shell.install_defaults();
    if let Err(e) = shell.run() {
        eprintln!("rsh: {e}");
        std::process::exit(1);
    }
}
