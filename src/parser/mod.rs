pub mod command;
pub mod parser;
pub mod pipeline;
pub mod redirect;

pub use command::Command;
pub use parser::Parser;
pub use pipeline::Pipeline;
pub use redirect::Redirect;



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    
    #[test]
    fn test_single_command() {
        let tokens = vec![Token::Word("ls".into())];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands[0].argv, vec!["ls"]);
    }

    #[test]
    fn test_command_with_args() {
        let tokens = vec![
            Token::Word("echo".into()),
            Token::Word("hello".into()),
        ];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands[0].argv, vec!["echo", "hello"]);
    }

    #[test]
    fn test_background() {
        let tokens = vec![Token::Word("sleep".into()), Token::Ampersand];
        let p = Parser::new(tokens).parse();
        assert!(p.background);
    }

    #[test]
    fn test_pipe() {
        let tokens = vec![
            Token::Word("ls".into()),
            Token::Pipe,
            Token::Word("grep".into()),
        ];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands.len(), 2);
    }

    #[test]
    fn test_redirect_in() {
        let tokens = vec![Token::Word("cat".into()), Token::RedirectIn, Token::Word("file.txt".into())];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands[0].stdin, Redirect::File("file.txt".into()));
    }

    #[test]
    fn test_redirect_out() {
        let tokens = vec![Token::Word("echo".into()), Token::RedirectOut, Token::Word("file.txt".into())];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands[0].stdout, Redirect::File("file.txt".into()));
    }

    #[test]
    fn test_redirect_append() {
        let tokens = vec![Token::Word("echo".into()), Token::RedirectAppend, Token::Word("file.txt".into())];
        let p = Parser::new(tokens).parse();
        assert_eq!(p.commands[0].stdout, Redirect::Append("file.txt".into()));
    }
}