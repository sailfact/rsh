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

#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    File(String),
    Inherit, 
    Pipe,
}
