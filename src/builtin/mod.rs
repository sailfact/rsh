pub mod cd;
pub mod alias;
pub mod ps;
pub mod exit;
pub mod ls;
pub mod echo;
pub mod pwd;
pub mod mkdir;
pub mod rm;

pub fn dispatch(name: &str, args:&[String], shell: &mut crate::Shell) -> i32 {
    match name {
        "cd" => cd::run(args, shell),
        "alias" => alias::run(args, shell),
        "ps" => ps::run(args, shell),
        "exit" => exit::run(args, shell),
        "ls" => ls::run(args, shell),
        "echo" => echo::run(args, shell),
        "pwd" => pwd::run(args, shell),
        "mkdir" => mkdir::run(args, shell),
        "rm" => rm::run(args, shell),
        _ => 1,
    }
}