// File: src/lib.rs
// Author: Ross Curley
// Repo: https://github.com/sailfact/rust-shell.git

// Module declarations
pub mod shell;
pub mod repl;
pub mod executor;
pub mod lexer;
pub mod parser;
pub mod jobs;
pub mod builtins;

// RE-exports
pub use shell::Shell;

pub use lexer::Lexer;
pub use lexer::token::Token;

pub use parser::Parser;
pub use parser::pipeline::Pipeline;
pub use parser::command::Command;
pub use parser::redirect::Redirect;

pub use jobs::Job;
pub use jobs::JobStatus;
pub use jobs::process::{Process, ProcessStatus};