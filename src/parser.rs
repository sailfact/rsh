pub struct Parser {
    tokens: Vec<Token>,
    parse: Pipeline,
}

pub struct Pipeline {
    commands: Vec<Command>,
    background: bool
}

pub struct Command {
    argv: Vec<string>,
    stdin: Redirect,
    stdout: Redirect
}

impl Command {
    pub fn is_builtin() -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    File(String),
    Inherit, 
    Pipe,
}

// pub fn rsh_split_line(line: &str) -> Vec<String> {
//     line.split(DELIMS)
//         .filter(|s| !s.is_empty()) 
//         .map(|s: &str| s.to_string())
//         .collect() 
// }
