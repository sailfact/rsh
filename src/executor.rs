pub fn rsh_execute(args: Vec<String>) -> i32 {
    if args.is_empty() {
        return 1;
    }

    if BUILTINS.iter().any(|(name, _)| *name == args[0]) {
        run_builtin(&args);
    }
    return rsh_launch(&args);
}



pub fn rsh_launch(args: &[String]) -> i32 {
    if args.is_empty(){ 
        return 1;
    }
    let mut child: Child = Command::new(&args[0])
        .args(&args[1..])
        .spawn()
        .expect("failed to lauch process⛔");
    let status = child.wait().expect("failed to wait process ⛔");

    status.code().unwrap_or(1)
}

