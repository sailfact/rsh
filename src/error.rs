use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplError {
    #[error("readline error: {0}")]
    Readline(#[from] rustyline::error::ReadlineError),

    #[error("failed to load history from {path}: {source}")]
    LoadHistory {
        path: String,
        #[source]
        source: rustyline::error::ReadlineError,
    },

    #[error("failed to save history to {path}: {source}")]
    SaveHistory {
        path: String,
        #[source]
        source: rustyline::error::ReadlineError,
    },
}

pub enum ReadResult {
    Line(String),
    Interrupted,
    Eof,
}