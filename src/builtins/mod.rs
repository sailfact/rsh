pub mod rshell;
pub mod coreutils;

use crate::Shell;

pub const RSHELL_BUILTINS: &[&str] = &[
    "cd", "exit", "alias", "unalias",
    "export", "unset", "source", ".",
    "jobs", "fg", "bg", "wait",
    "trap", "umask", "read", "type",
    "pwd", "ps", "history", "hash",
    "true", "false", "shift",
    "break", "continue", "return",
    "kill",  
];

pub const UUTILS_BUILTINS: &[&str] = &[
    // file ops
    "cp", "mv", "rm", "mkdir", "rmdir", "touch", "ln",
    "chmod", "chown", "stat", "du", "df", "install", "mktemp",
    // text
    "cat", "echo", "printf", "head", "tail", "wc", "sort",
    "uniq", "cut", "tr", "paste", "join", "fold", "fmt",
    "nl", "tac", "rev", "expand", "unexpand", "od", "xxd", "grep",
    // system
    "whoami", "id", "hostname", "uname", "uptime", "date",
    "sleep", "yes", "env", "nohup", "timeout", "nice",
    "printenv", "tty", "stty",
    // hashing
    "base32", "base64", "md5sum", "sha256sum",
    "sha512sum", "cksum", "sum", "b2sum",
    // misc
    "seq", "factor", "basename", "dirname", "realpath",
    "pathchk", "link", "unlink", "sync", "truncate",
    "shuf", "comm", "csplit", "split", "tee",
    "test", "[", "expr", "xargs",
];


pub fn dispatch(name: &str, args:&[String], shell: &mut Shell) -> i32 {
    match name {
        "cd"      => cd::Cd.run(args, shell),
        "exit"    => exit::Exit.run(args, shell),
        "alias"   => alias::Alias.run(args, shell),
        "export"   => export::Export.run(args, shell),
        "jobs"      => ps::Ps.run(args, shell),
        "pwd"     => pwd::Pwd.run(args, shell),
        "fg"      => fg::Fg.run(args, shell),
        "bg"      => bg::Bg.run(args, shell),
        "ls"      => ls::Ls.run(args, shell),
        "echo"    => echo::Echo.run(args, shell),
        "mkdir"   => mkdir::Mkdir.run(args, shell),
        "rm"      => rm::Rm.run(args, shell),
        _         => 1,
    }
}

pub fn is_shell_builtin(name: &str) -> bool {
    matches!(name, "cd" | "alias" | "ps" | "exit" | "ls" | "echo" | "pwd" | "mkdir" | "rm" | "fg" | "bg")
}

pub fn is_uutils_builtin(name: &str) -> bool {
    coreutils::ALL_COMMANDS.iter().any(|group| group.contains(&name))
}
