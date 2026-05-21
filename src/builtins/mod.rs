// src/builtins/mod.rs
pub mod rshell;
pub mod coreutils;

use crate::ast::Command;
use crate::shell::Shell;

// ── Lookup tables ─────────────────────────────────────────────────────────────
// These are the single source of truth for command routing.
// executor.rs calls is_rshell_builtin() / is_uutils_builtin() to decide
// whether to fork and which dispatcher to call.

pub const SHELL_BUILTINS: &[&str] = &[
    "cd", "exit", "exec",
    "alias", "unalias", "export", "unset", "set", "shift",
    "source", ".", "return", "break", "continue",
    "jobs", "fg", "bg", "wait", "kill",
    "trap", "umask", "read", "pwd",
    "echo", "true", "false",
    "type", "hash", "history",
];

pub const UUTILS_BUILTINS: &[&str] = &[
    // file ops
    "cp", "mv", "rm", "mkdir", "rmdir", "touch", "ln",
    "chmod", "chown", "stat", "du", "df", "install", "mktemp",
    // text
    "cat", "printf", "head", "tail", "wc", "sort",
    "uniq", "cut", "tr", "paste", "join", "fold", "fmt",
    "tac", 
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
    "test", "[", "expr",
];

// ── Routing ───────────────────────────────────────────────────────────────────

pub fn is_rshell_builtin(name: &str) -> bool {
    SHELL_BUILTINS.contains(&name)
}

pub fn is_uutils_builtin(name: &str) -> bool {
    UUTILS_BUILTINS.contains(&name)
}

pub fn is_any_builtin(name: &str) -> bool {
    is_rshell_builtin(name) || is_uutils_builtin(name)
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
// These are thin wrappers — all real logic lives in the submodules.
// executor.rs only ever calls these two functions.

/// Called in the SHELL process (before fork).
/// Handles trivial commands inline, then delegates to the Registry.
pub fn exec_shell_builtin(cmd: &Command, shell: &mut Shell) -> i32 {
    // Trivial commands — not worth a Registry entry
    match cmd.argv[0].as_str() {
        "true"     => return 0,
        "false"    => return 1,
        "break"    => return 128,  
        "continue" => return 129,  
        _ => {}
    }

    // Everything else goes through the Registry
    rshell::dispatch(cmd, shell)
}

/// Called in the CHILD process (after fork + dup2).
/// Delegates entirely to the coreutils dispatcher.
pub fn exec_uutils_builtin(cmd: &Command) -> i32 {
    coreutils::dispatch(cmd)
}