use crate::lexer::Token;
use super::Pipeline;
use super::Command;
use super::Redirect;

pub struct Parser {
    tokens: Vec<Token>,
}

// imple Parser
impl Parser{
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens: tokens }
    }
    pub fn parse(&mut self) -> Pipeline {
        let mut cmds: Vec<Command> = Vec::new();
        let mut bg = false;
        // strip trailing '&' before parsing
        if self.tokens.last() == Some(&Token::Ampersand) {
            bg = true;
            self.tokens.pop();
        }

        // split token stream on Pipe boundaries into per-command slices
        let segments: Vec<Vec<Token>> = self.tokens
            .split(|t| t == &Token::Pipe)
            .map(|s| s.to_vec())
            .collect();
        let segment_count = segments.len();

        for (i, segment) in segments.into_iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == segment_count - 1;
            cmds.push(self.parse_command(segment, is_first, is_last));
        }

        Pipeline::new(cmds, bg)
    }

    fn parse_command(&self, tokens: Vec<Token>, is_first: bool, is_last: bool) -> Command {
        let mut argv: Vec<String> = Vec::new();
        let mut stdin  = if is_first { Redirect::Inherit } else { Redirect::Pipe };
        let mut stdout = if is_last  { Redirect::Inherit } else { Redirect::Pipe };

        let mut iter = tokens.into_iter().peekable();

        while let Some(token) = iter.next() {
            match token {
                Token::Word(w) => argv.push(w),

                Token::RedirectIn => {
                    if let Some(Token::Word(path)) = iter.next() {
                        stdin = Redirect::File(path);
                    }
                }

                Token::RedirectOut => {
                    if let Some(Token::Word(path)) = iter.next() {
                        stdout = Redirect::File(path);
                    }
                }

                Token::RedirectAppend => {
                    if let Some(Token::Word(path)) = iter.next() {
                        stdout = Redirect::Append(path);
                    }
                }

                // Pipe and Ampersand are handled at the pipeline level,
                // so hitting one here means a malformed token stream
                _ => {}
            }
        }

        Command::new(argv, stdin, stdout)
    }
}