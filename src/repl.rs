use rustyline::Editor;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Helper, Highlighter, Hinter, Validator};
use std::path::PathBuf;
use thiserror::Error;

/// Line-editing helper: command-name completion in command position,
/// filename completion everywhere else. Hinting/highlighting/validation
/// are no-op derives.
#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct RshHelper {
    filename: FilenameCompleter,
    commands: Vec<String>,
}

impl RshHelper {
    fn new() -> Self {
        let mut commands: Vec<String> = crate::builtins::SHELL_BUILTINS
            .iter()
            .chain(crate::builtins::UUTILS_BUILTINS.iter())
            .map(|s| s.to_string())
            .collect();

        // Every executable on PATH is a completion candidate.
        if let Ok(path) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path) {
                let Ok(entries) = std::fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    if entry.path().is_file()
                        && let Some(name) = entry.file_name().to_str()
                    {
                        commands.push(name.to_string());
                    }
                }
            }
        }
        commands.sort();
        commands.dedup();
        Self {
            filename: FilenameCompleter::new(),
            commands,
        }
    }

    /// A word is in command position if everything before it (in the
    /// current pipeline segment) is blank or a command separator.
    fn is_command_position(before_word: &str) -> bool {
        matches!(
            before_word.trim_end().chars().last(),
            None | Some('|' | ';' | '&')
        )
    }
}

impl Completer for RshHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind([' ', '\t', '|', ';', '&', '<', '>'])
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line[start..pos];

        if Self::is_command_position(&line[..start]) && !word.contains('/') {
            let matches = self
                .commands
                .iter()
                .filter(|c| c.starts_with(word))
                .map(|c| Pair {
                    display: c.clone(),
                    replacement: c.clone(),
                })
                .collect();
            Ok((start, matches))
        } else {
            self.filename.complete(line, pos, ctx)
        }
    }
}

// REPL - Read, Eval, Print, Loop
// Read — wait for the user to type a line and press Enter
// Eval — parse and execute that line
// Print — show the output (or prompt for the next command)
// Loop — go back to the start
pub struct Repl {
    editor: Editor<RshHelper, FileHistory>,
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
        let mut editor: Editor<RshHelper, FileHistory> =
            Editor::new().map_err(ReplError::ReadLine)?;
        editor.set_helper(Some(RshHelper::new()));
        // Set prompt if none provided
        let prompt = if prompt.is_empty() {
            String::from("rsh> ")
        } else {
            prompt
        };

        Ok(Self {
            editor,
            history: None,
            prompt,
        })
    }
    pub fn with_history(mut self, path: &str) -> Self {
        let resolved = if let Some(rest) = path.strip_prefix("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
            PathBuf::from(format!("{}/{}", home, rest))
        } else {
            PathBuf::from(path)
        };

        match self.editor.load_history(&resolved) {
            Ok(_) => {}
            Err(ReadlineError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
                // First run — create the file so save_history() works later
                if let Some(parent) = resolved.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::File::create(&resolved) {
                    eprintln!("rsh: warning: could not create history file: {e}");
                }
            }
            Err(e) => {
                eprintln!("rsh: warning: could not load history: {e}"); // ← missing
            }
        }
        self.history = Some(resolved);
        self
    }

    pub fn read_line(&mut self) -> Result<ReadResult, ReplError> {
        match self.editor.readline(&self.prompt) {
            Ok(line) => Ok(ReadResult::Line(line)),
            Err(ReadlineError::Interrupted) => Ok(ReadResult::Interrupted),
            Err(ReadlineError::Eof) => Ok(ReadResult::Eof),
            Err(e) => Err(ReplError::ReadLine(e)),
        }
    }

    pub fn add_history(&mut self, line: &str) {
        let _ = self.editor.add_history_entry(line);
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    pub fn save_history(&mut self) -> Result<(), ReplError> {
        if let Some(ref path) = self.history {
            self.editor
                .save_history(path)
                .map_err(|e| ReplError::SaveHistory {
                    path: path.display().to_string(),
                    source: e,
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete(line: &str) -> (usize, Vec<Pair>) {
        let helper = RshHelper::new();
        let history = FileHistory::new();
        let ctx = rustyline::Context::new(&history);
        helper.complete(line, line.len(), &ctx).unwrap()
    }

    #[test]
    fn completes_builtin_in_command_position() {
        let (start, candidates) = complete("ech");
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|p| p.replacement == "echo"));
    }

    #[test]
    fn completes_command_after_pipe_and_semicolon() {
        let (start, candidates) = complete("ls | ech");
        assert_eq!(start, 5);
        assert!(candidates.iter().any(|p| p.replacement == "echo"));

        let (_, candidates) = complete("true; expor");
        assert!(candidates.iter().any(|p| p.replacement == "export"));
    }

    #[test]
    fn argument_position_completes_filenames() {
        // Run from the crate root: Cargo.toml exists.
        let (_, candidates) = complete("cat Cargo.to");
        assert!(
            candidates
                .iter()
                .any(|p| p.replacement.contains("Cargo.toml")),
            "expected Cargo.toml in {:?}",
            candidates
                .iter()
                .map(|p| &p.replacement)
                .collect::<Vec<_>>()
        );
    }
}
