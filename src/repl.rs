use std::path::PathBuf;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use thiserror::Error;

pub struct Repl{
    editor: DefaultEditor,
    history: Option<PathBuf>,
    prompt: String,
}


#[derive(Debug, Error)]
pub enum ReplError {
    #[error("readline error: {0}")]
    ReadLine(#[from] rustyline::error::ReadlineError),

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

impl Repl {
    pub fn new(prompt: String) -> Result<Self, ReplError> {
        // Create Default Editor
        let editor = DefaultEditor::new().map_err(ReplError::ReadLine)?;
        // Set prompt if none provided
        let prompt = if prompt.is_empty() { String::from(">>>") } else { prompt };
        
        Ok(Self {
            editor,
            history: None,
            prompt,
        })
    }
    pub fn with_history(mut self, path: &str) -> Self {
        let resolved = if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
            PathBuf::from(format!("{}/{}", home, &path[2..]))
        } else {
            PathBuf::from(path)
        };
        let pb = resolved;
        match self.editor.load_history(&pb) {
            Ok(_) => {}
            Err(ReadlineError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
                // First run — create the file so save_history() works later
                if let Some(parent) = pb.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::File::create(&pb) {
                    eprintln!("rsh: warning: could not create history file: {e}");
                }
            }
            Err(e) => {
                eprintln!("rsh: warning: could not load history: {e}");  // ← missing
            }
        }
        self.history = Some(pb);
        self 
    }

    pub fn read_line(&mut self) -> Result<ReadResult, ReplError> {
        match self.editor.readline(&self.prompt) {
            Ok(line) => {
                let _ = self.editor.add_history_entry(&line);
                Ok(ReadResult::Line(line))
            }
            Err(ReadlineError::Interrupted) => Ok(ReadResult::Interrupted),
            Err(ReadlineError::Eof) => Ok(ReadResult::Eof),
            Err(e) => Err(ReplError::ReadLine(e)),
        }
    }

    pub fn save_history(&mut self) -> Result<(), ReplError>{
        if let Some(ref path) = self.history {
            self.editor.save_history(path).map_err(|e| ReplError::SaveHistory {
                path: path.display().to_string(),
                source: e,
            })?;
        }
        Ok(())
    }
}  