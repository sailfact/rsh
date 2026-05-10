#[derive(Debug, PartialEq, Clone)]
pub enum Token{
    Word(String),
    Pipe,
    RedirectIn,
    RedirectOut, 
    RedirectAppend, 
    Ampersand, 
    Semicolon,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    // Create new lexer from a give input
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect,
            pos: 0,
        }
    }

    // Return character at current position
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    // Return next character and increment position
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        self.pos +=1;
        ch
    }
}

impl Lexer {
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            // Get the next character
            match ch {
                ' ' | '\t' => { self.advance(); } // Skip Spaces ' ' & tabs '\t'
                '|' => { self.advance(); tokens.push(Token::Pipe); } // Push | on to heap
                '&' => { self.advance(); tokens.push(Token::Ampersand); } // Push & on to heap
                ';' => { self.advance(); tokens.push(Token::Semicolon); } // Push ; on to heap
                '<' => { self.advance(); tokens.push(Token::RedirectIn); } // Push < on to heap
                '>' => { 
                    self.advance(); 
                    if self.peek() == Some('>'){ // check if next character forms an append redirect
                        self.advance();
                        tokens.push(Token::RedirectAppend);
                    } else { // push redirect out
                        tokens.push(Token::RedirectOut);
                    }
                } 
                '\'' | '"' => { // quoted string
                    let word = self.read_quoted();
                    tokens.push(Token::Word(word));
                }
                // All other values are a work 
                _ => {
                    let word = self.read_word();
                    tokens.push(Token::Word(word));
                }
            }
        }
        tokens
    }
}

impl Lexer {
    // Read an unquoted word — stops at whitespace or an operator character.
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '|' | '&' | ';' | '<' | '>' | '\'' | '"' => break,
                _ => { word.push(ch); self.advance(); }
            }
        }
        word
    }

    // Read a single- or double-quoted string, consuming the surrounding quotes.
    fn read_quoted(&mut self) -> String {
        let quote = self.advance().unwrap(); // consume opening quote
        let mut word = String::new();
        while let Some(ch) = self.advance() {
            if ch == quote { break; }
            word.push(ch);
        }
        word
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer::new(input).tokenize()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let tokens = tokenize("ls -la");
        assert_eq!(tokens, vec![
            Token::Word("ls".into()),
            Token::Word("-la".into()),
        ]);
    }

    #[test]
    fn test_pipe() {
        let tokens = tokenize("ls | grep foo");
        assert_eq!(tokens, vec![
            Token::Word("ls".into()),
            Token::Pipe,
            Token::Word("grep".into()),
            Token::Word("foo".into()),
        ]);
    }

    #[test]
    fn test_redirect_append() {
        let tokens = tokenize("echo hi >> out.txt");
        assert_eq!(tokens, vec![
            Token::Word("echo".into()),
            Token::Word("hi".into()),
            Token::RedirectAppend,
            Token::Word("out.txt".into()),
        ]);
    }

    #[test]
    fn test_quoted_string() {
        let tokens = tokenize("echo 'hello world'");
        assert_eq!(tokens, vec![
            Token::Word("echo".into()),
            Token::Word("hello world".into()),
        ]);
    }
}