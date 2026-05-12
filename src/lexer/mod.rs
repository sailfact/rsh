pub mod token;
pub mod lexer;

pub use token::Token;
pub use lexer::Lexer;

pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer::new(input).tokenize()
}

// Test Modules
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