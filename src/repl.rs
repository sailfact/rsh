use std::path::PathBuf;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub struct Repl{
    editor: DefaultEditor,
    history: Option<PathBuf>,
    label: String,
}

impl Repl {
    pub fn new(history: Option<&str>, label: String) -> Result<Self, ReplError> {
        // Create Default Editor
        let mut editor = DefaultEditor::new()
            .map_err(ReplError::ReadLine);
        // Set label if none provided
        let label = if label.is_empty() { String::from(">>>") } else { label };
        
        let mut path_buffer = None;

        // if history is give set History
        if let Some(path) = history {
            let _ = editor.load_history(path).map_err(|e| ReplError::LoadHistory {
                path: path.to_String(),
                source: e,
            })?;
            path_buffer = Some(PathBuf::from(path));
        }
        // Create Repl
        Ok(Self { 
            editor,
            history: path_buffer,
            label,
        })
    }

    pub fn read_line(&mut self) -> Result<ReadResult, ReadlineError> {
        match self.editor.readline(&self.label) {
            Ok(line) => {
                let _ = self.editor.add_history_entry(&line);
                Ok(ReadResult::ReadLine(line))
            }
            Err(ReadlineError::Interrupted) => Ok(ReadResult::Interrupt),
            Err(ReadlineError::Eof) => Ok(ReadResult::EoF),
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