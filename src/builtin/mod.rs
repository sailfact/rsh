pub mod cd;
pub mod alias;
pub mod ps;
pub mod exit;
pub mod ls;
pub mod echo;
pub mod pwd;
pub mod mkdir;
pub mod rm;
pub mod fg;
pub mod bg;

use crate::Shell;

pub trait Builtin {
    fn name(&self) -> &str;
    fn run(&self, args: &[String], shell: &mut Shell) -> i32;
}

pub fn dispatch(name: &str, args:&[String], shell: &mut Shell) -> i32 {
    let builtins: Vec<Box<dyn Builtin>> = vec![
        Box::new(cd::Cd),
        Box::new(alias::Alias),
        Box::new(alias::Unalias),
        Box::new(ps::Ps),
        Box::new(exit::Exit),
        Box::new(ls::Ls),
        Box::new(echo::Echo),
        Box::new(pwd::Pwd),
        Box::new(mkdir::Mkdir),
        Box::new(rm::Rm),
        Box::new(fg::Fg),
        Box::new(bg::Bg),
    ];
    match builtins.iter().find(|b| b.name() == name) {
        Some(builtin) => builtin.run(args, shell),
        None => 1,
    }
}

pub fn is_builtin(name: &str) -> bool {
    matches!(name, "cd" | "alias" | "ps" | "exit" | "ls" | "echo" | "pwd" | "mkdir" | "rm")
}