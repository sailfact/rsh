use std::ffi::CString;
use nix::unistd::execvp;
use crate::ast::command::Command;

pub fn exec(cmd: &Command) -> ! {
    let name = CString::new(cmd.argv[0].as_str()).expect("CString::new");
    let args: Vec<CString> = cmd.argv.iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new"))
        .collect();

    match execvp(&name, &args) {
        Ok(v)  => match v {},
        Err(e) => {
            eprintln!("rsh: {}: {}", cmd.argv[0], e);
            std::process::exit(127);
        }
    }
}
