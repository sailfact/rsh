// File: src/lib.rs
// Author: Ross Curley
// Repo: https://github.com/sailfact/rsh.git

pub mod ast;
pub mod builtins;
pub mod executor;
pub mod expansion;
pub mod external;
pub mod jobs;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod script;
pub mod shell;
pub mod signals;

pub use repl::Repl;
pub use repl::{ReadResult, ReplError};
pub use shell::Shell;

// builtins module

// lexer module
pub use lexer::lexer::Lexer;
pub use lexer::token::Token;
pub use lexer::tokenize;

// parser module
pub use parser::parser::Parser;

// ast module
pub use ast::command::Command;
pub use ast::pipeline::Pipeline;
pub use ast::redirect::Redirect;

// jobs module
pub use jobs::Job;
pub use jobs::JobStatus;
pub use jobs::process::{Process, ProcessStatus};
