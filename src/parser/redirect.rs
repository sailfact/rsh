#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    File(String),
    Inherit, 
    Pipe,
    Append(String),
}
