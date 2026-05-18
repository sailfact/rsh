use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Hash;

impl Builtin for Hash {
    fn name(&self) -> &'static str { "hash" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("hash not implemented");
        0
    }
}