use std::env;

// Builtin command function type
pub type BuiltinFn = fn(&[String]) -> i32;

// Pair up command names with their functions
pub const BUILTINS: &[(&str, BuiltinFn)] = &[
    ("cd",   rsh_cd),
    ("help", rsh_help),
    ("exit", rsh_exit),
    ("alias", rsh_help),
    ("jobs", rsh_jobs),
    ("fg", rsh_fg),
    ("bg", rsh_bg),
];

// Look up and run a builtin, returns None if not a builtin
pub fn run_builtin(args: &[String]) -> Option<i32> {
    let cmd = args.first()?;
    let (_, func) = BUILTINS.iter().find(|(name, _)| name == cmd)?;
    Some(func(args))
}

pub fn rsh_cd(args: &[String]) -> i32 {
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

pub fn rsh_help(_args: &[String]) -> i32 {
    println!("Rust Shell (rsh)");
    println!("Type program names and arguments, and hit enter.");
    println!("The following are built in:");
    for (name, _) in BUILTINS {
        println!("  {name}");
    }
    println!("Use the man command for information on other programs.");
    1
}

pub fn rsh_exit(_args: &[String]) -> i32 {
    0
}