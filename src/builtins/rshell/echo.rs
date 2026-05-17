use crate::shell::Shell;
use super::Builtin;

pub struct Echo;    

impl Builtin for Echo {
    fn name(&self) -> &str { "echo" }

    fn run(&self, args: &[String], _shell: &mut Shell) -> i32 {
        let output = args.join(" ");
        println!("{}", output);
        0
    }
    
}