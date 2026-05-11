#[derive(Debug, PartialEq, Clone)]
pub enum Token{
    Word(String),
    Pipe,
    RedirectIn,
    RedirectOut, 
    RedirectAppend, 
    Ampersand, 
    Semicolon,
}

