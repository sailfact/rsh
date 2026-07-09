use rsh::Shell;

fn main() {
    let mut shell = Shell::new();
    if shell.is_interactive {
        shell.install_defaults();
    }
    match shell.run() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("rsh: {e}");
            std::process::exit(1);
        }
    }
}
