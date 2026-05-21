// builtin/coreutils/mod.rs
// Delegates core util commands
pub mod file_util;
pub mod hash_util;
pub mod misc_util;
pub mod system_util;
pub mod text_util;

use crate::Command;
use std::ffi::OsString;

// Flat list for builtin/mod.rs to use in is_uutils_builtin()
pub const ALL_COMMANDS: &[&[&str]] = &[
    file_util::COMMANDS,
    text_util::COMMANDS,
    system_util::COMMANDS,
    hash_util::COMMANDS,
    misc_util::COMMANDS,
];

pub fn dispatch(cmd: &Command) -> i32 {
    let args: Vec<OsString> = cmd.argv.iter().map(OsString::from).collect();
    let name = cmd.argv[0].as_str();

    if file_util::COMMANDS.contains(&name) {
        return file_util::dispatch(name, args);
    }
    if text_util::COMMANDS.contains(&name) {
        return text_util::dispatch(name, args);
    }
    if system_util::COMMANDS.contains(&name) {
        return system_util::dispatch(name, args);
    }
    if hash_util::COMMANDS.contains(&name) {
        return hash_util::dispatch(name, args);
    }
    if misc_util::COMMANDS.contains(&name) {
        return misc_util::dispatch(name, args);
    }

    eprintln!("shell: unknown coreutils command: {}", name);
    1
}
