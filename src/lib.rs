// File: src/lib.rs
// Author: Ross Curley
// Repo: https://github.com/sailfact/rsh.git

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod builtins;
pub mod shell;
pub mod repl;
pub mod executor;
pub mod external;
pub mod jobs;

pub use shell::Shell;
pub use repl::Repl;
pub use repl::{ReadResult, ReplError};

// builtins module

// lexer module
pub use lexer::lexer::Lexer;
pub use lexer::token::Token;
pub use lexer::tokenize;

// parser module
pub use parser::parser::Parser;

// ast module
pub use ast::pipeline::Pipeline;
pub use ast::command::Command;
pub use ast::redirect::Redirect;

// jobs module
pub use jobs::Job;
pub use jobs::JobStatus;
pub use jobs::process::{Process, ProcessStatus};