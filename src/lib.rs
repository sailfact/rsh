// File: src/lib.rs
// Author: Ross Curley
// Repo: https://github.com/sailfact/rsh.git

pub mod shell;
pub mod repl;
pub mod executor;
pub mod parser;
pub mod lexer;
pub mod jobs;
pub mod builtin;

pub use shell::Shell;
pub use repl::Repl;
pub use repl::{ReadResult, ReplError};

// lexer module
pub use lexer::lexer::Lexer;
pub use lexer::token::Token;

// parser module
pub use parser::parser::Parser;
pub use parser::pipeline::Pipeline;
pub use parser::command::Command;
pub use parser::Redirect;

pub use jobs::Job;
pub use jobs::JobStatus;
pub use jobs::process::{Process, ProcessStatus};