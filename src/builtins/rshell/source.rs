use crate::shell::Shell;
use super::Builtin;
use std::env;

pub struct Source;
pub struct Dot;
pub struct Return;

impl Builtin for Source {
    fn name(&self) -> &str { "source" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("source not implemented");
        0
    }
}

impl Builtin for Dot{
    fn name(&self) -> &str { "." }
    
    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("source not implemented");
        0
    }
}

impl Builtin for Return{
    fn name(&self) -> &str { "return" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        // Todo
        println!("source not implemented");
        0
    }
}