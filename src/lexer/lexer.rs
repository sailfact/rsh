use super::Token;

// Lexer
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    // Create new lexer from a give input
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
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
        self.pos += 1;
        ch
    }
}

impl Lexer {
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            // Get the next character
            match ch {
                ' ' | '\t' => {
                    self.advance();
                } // Skip Spaces ' ' & tabs '\t'
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        tokens.push(Token::OrIf);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        tokens.push(Token::AndIf);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }
                ';' => {
                    self.advance();
                    tokens.push(Token::Semicolon);
                } // Push ; on to heap
                '<' => {
                    self.advance();
                    tokens.push(Token::RedirectIn);
                } // Push < on to heap
                '>' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        // check if next character forms an append redirect
                        self.advance();
                        tokens.push(Token::RedirectAppend);
                    } else {
                        // push redirect out
                        tokens.push(Token::RedirectOut);
                    }
                }
                _ => {
                    let word = self.read_word();
                    tokens.push(Token::Word(word));
                }
            }
        }
        tokens
    }
    // Read a word — unquoted and quoted runs with no intervening whitespace
    // glue together into a single token (so `ll='ls -la'` is one word).
    //
    // Quote characters are KEPT in the token: the expansion pass
    // (src/expansion.rs) needs them to decide what to expand, and it is
    // responsible for quote removal.
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '|' | '&' | ';' | '<' | '>' => break,
                '\'' | '"' => word.push_str(&self.read_quoted()),
                '$' => {
                    word.push(ch);
                    self.advance();
                    // `$(cmd ...)` and `$((expr ...))` may contain spaces
                    // and operators; consume through the balanced close so
                    // the whole construct stays in one word.
                    if self.peek() == Some('(') {
                        word.push_str(&self.read_balanced_parens());
                    }
                }
                _ => {
                    word.push(ch);
                    self.advance();
                }
            }
        }
        word
    }

    // Consume a parenthesized run starting at `(`, through its balanced
    // closing paren, including everything (whitespace, operators) inside.
    fn read_balanced_parens(&mut self) -> String {
        let mut content = String::new();
        let mut depth = 0u32;
        while let Some(ch) = self.advance() {
            content.push(ch);
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        content
    }

    // Read a single- or double-quoted string, including the surrounding
    // quotes (quote removal happens during expansion).
    fn read_quoted(&mut self) -> String {
        let quote = self.advance().unwrap(); // consume opening quote
        let mut word = String::new();
        word.push(quote);
        while let Some(ch) = self.advance() {
            word.push(ch);
            if ch == quote {
                break;
            }
        }
        word
    }
}
