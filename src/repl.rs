use std::path::PathBuf;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustyline::Result;

pub struct Repl{
    editor: DefaultEditor,
    history: Option<PathBuf>,
    prompt: &str,
}

impl Repl {
    pub fn new(history: Option<&str>, prompt: &str) -> Result<Self> {
        let mut editor = DefaultEditor::new()?;
        let mut path_buffer = None;
        if prompt.isEmpty() {
            let prompt = ">>>";
        }


        if let Some(path) = history {
            let _ = editor.load_history(path);
            path_buffer = Some(PathBuf::from(path));
        }
        Ok(Self { 
            editor,
            history: path_buffer,
            prompt: prompt,
        })
    }

    fn read_line(&mut self, label: &str) -> Result<()> {
        loop {
            match self.editor.readline(&self.prompt) {
                Ok(line) => {
                    self.add_history(line.as_str())?;
                    println!("Line: {}", line);
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                },
                    Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        self.save_history(&line)
    }


    fn add_history(&mut self, line: &str) {
        self.editor.add_history_entry(&line);
    }

    fn save_history(&mut self, history: Option<&str>) {
        self.editor.save_history(history);
    }
}

pub fn prompt(history: Option<&str>, label: &str) -> i32 {
    Repl::new(history, &str);
    0
}