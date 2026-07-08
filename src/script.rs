//! Script-level grammar: compound commands above the pipeline level.
//!
//! `Shell::eval` lexes the whole input (newlines become `;`), this module
//! parses the token stream into a recursive [`Ast`] and executes it.
//! Supported constructs:
//!
//! ```text
//! if LIST; then LIST; [elif LIST; then LIST;]... [else LIST;] fi
//! while LIST; do LIST; done          until LIST; do LIST; done
//! for NAME in WORDS...; do LIST; done
//! { LIST; }                          NAME() { LIST; }
//! break [n] / continue [n] / return [n]
//! ```
//!
//! Keywords are only recognized unquoted at command position, so
//! `echo done` still works. The parser distinguishes "input ended
//! mid-construct" ([`ParseError::Incomplete`]) from real syntax errors so
//! the interactive REPL can prompt for continuation lines.

use crate::lexer::Token;
use crate::shell::Shell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sep {
    Seq,        // `;` or newline
    Background, // `&`
    And,        // `&&`
    Or,         // `||`
}

#[derive(Debug, Clone)]
pub enum Ast {
    /// A simple pipeline, kept as raw tokens; expansion happens at exec time.
    Pipeline(Vec<Token>),
    List(Vec<(Ast, Sep)>),
    If {
        cond: Box<Ast>,
        then_body: Box<Ast>,
        elifs: Vec<(Ast, Ast)>,
        else_body: Option<Box<Ast>>,
    },
    Loop {
        until: bool,
        cond: Box<Ast>,
        body: Box<Ast>,
    },
    For {
        var: String,
        words: Vec<String>,
        body: Box<Ast>,
    },
    FuncDef {
        name: String,
        body: Box<Ast>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The input ended inside a construct — more lines may complete it.
    Incomplete,
    Syntax(String),
}

/// How execution left an AST node: normally with a status, or via a loop /
/// function control transfer that outer nodes must handle or propagate.
pub enum Flow {
    Normal(i32),
    Break(u32),
    Continue(u32),
    Return(i32),
}

/// Keywords that terminate an inner list; stray occurrences at command
/// position are syntax errors.
const RESERVED: &[&str] = &["then", "elif", "else", "fi", "do", "done", "}"];

// ── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParseError> {
    let mut parser = Parser { tokens, pos: 0 };
    let ast = parser.parse_list(&[])?;
    if parser.pos < parser.tokens.len() {
        return Err(ParseError::Syntax(format!(
            "unexpected token near {:?}",
            parser.tokens[parser.pos]
        )));
    }
    Ok(ast)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn cur(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn cur_word(&self) -> Option<&str> {
        match self.cur() {
            Some(Token::Word(w)) => Some(w.as_str()),
            _ => None,
        }
    }

    fn skip_separators(&mut self) {
        while matches!(self.cur(), Some(Token::Semicolon)) {
            self.pos += 1;
        }
    }

    /// Consume the keyword `kw` (possibly after separators). EOF here means
    /// the construct can still be completed by more input.
    fn expect_keyword(&mut self, kw: &str) -> Result<(), ParseError> {
        self.skip_separators();
        match self.cur_word() {
            Some(w) if w == kw => {
                self.pos += 1;
                Ok(())
            }
            None => Err(ParseError::Incomplete),
            Some(w) => Err(ParseError::Syntax(format!(
                "expected '{}', found '{}'",
                kw, w
            ))),
        }
    }

    /// Parse a command list until EOF or one of `terminators` appears at
    /// command position (the terminator is not consumed).
    fn parse_list(&mut self, terminators: &[&str]) -> Result<Ast, ParseError> {
        let mut items: Vec<(Ast, Sep)> = Vec::new();

        loop {
            self.skip_separators();
            if self.cur().is_none() {
                break;
            }
            if let Some(w) = self.cur_word()
                && terminators.contains(&w)
            {
                break;
            }

            let item = self.parse_item(terminators)?;

            let sep = match self.cur() {
                Some(Token::Semicolon) => {
                    self.pos += 1;
                    Sep::Seq
                }
                Some(Token::Ampersand) => {
                    self.pos += 1;
                    Sep::Background
                }
                Some(Token::AndIf) => {
                    self.pos += 1;
                    Sep::And
                }
                Some(Token::OrIf) => {
                    self.pos += 1;
                    Sep::Or
                }
                _ => Sep::Seq,
            };
            items.push((item, sep));

            // `a &&` at end of input: the right-hand side may arrive on the
            // next line (newlines are allowed after && and ||).
            if matches!(sep, Sep::And | Sep::Or) {
                self.skip_separators();
                if self.cur().is_none() {
                    return Err(ParseError::Incomplete);
                }
            }
        }

        Ok(Ast::List(items))
    }

    fn parse_item(&mut self, _terminators: &[&str]) -> Result<Ast, ParseError> {
        match self.cur_word() {
            Some("if") => self.parse_if(),
            Some("while") => self.parse_loop(false),
            Some("until") => self.parse_loop(true),
            Some("for") => self.parse_for(),
            Some("{") => self.parse_group(),
            Some(w) if RESERVED.contains(&w) => {
                Err(ParseError::Syntax(format!("unexpected '{}'", w)))
            }
            Some(w) if is_funcdef_word(w) => self.parse_funcdef(),
            _ => self.parse_pipeline(),
        }
    }

    /// Collect raw tokens of one pipeline, up to (not including) a list
    /// separator.
    fn parse_pipeline(&mut self) -> Result<Ast, ParseError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.cur() {
            match token {
                Token::Semicolon | Token::Ampersand | Token::AndIf | Token::OrIf => break,
                _ => {
                    tokens.push(token.clone());
                    self.pos += 1;
                }
            }
        }
        Ok(Ast::Pipeline(tokens))
    }

    fn parse_if(&mut self) -> Result<Ast, ParseError> {
        self.pos += 1; // "if"
        let cond = Box::new(self.parse_list(&["then"])?);
        self.expect_keyword("then")?;
        let then_body = Box::new(self.parse_list(&["elif", "else", "fi"])?);

        let mut elifs = Vec::new();
        while self.peek_keyword() == Some("elif") {
            self.pos += 1;
            let c = self.parse_list(&["then"])?;
            self.expect_keyword("then")?;
            let b = self.parse_list(&["elif", "else", "fi"])?;
            elifs.push((c, b));
        }

        let else_body = if self.peek_keyword() == Some("else") {
            self.pos += 1;
            Some(Box::new(self.parse_list(&["fi"])?))
        } else {
            None
        };

        self.expect_keyword("fi")?;
        Ok(Ast::If {
            cond,
            then_body,
            elifs,
            else_body,
        })
    }

    fn parse_loop(&mut self, until: bool) -> Result<Ast, ParseError> {
        self.pos += 1; // "while" / "until"
        let cond = Box::new(self.parse_list(&["do"])?);
        self.expect_keyword("do")?;
        let body = Box::new(self.parse_list(&["done"])?);
        self.expect_keyword("done")?;
        Ok(Ast::Loop { until, cond, body })
    }

    fn parse_for(&mut self) -> Result<Ast, ParseError> {
        self.pos += 1; // "for"
        let var = match self.cur_word() {
            Some(w) if is_valid_name(w) => {
                let v = w.to_string();
                self.pos += 1;
                v
            }
            None => return Err(ParseError::Incomplete),
            Some(w) => {
                return Err(ParseError::Syntax(format!(
                    "'{}' is not a valid for-loop variable",
                    w
                )));
            }
        };

        match self.cur_word() {
            Some("in") => self.pos += 1,
            None => return Err(ParseError::Incomplete),
            _ => {
                return Err(ParseError::Syntax(
                    "expected 'in' after for-loop variable".into(),
                ));
            }
        }

        // Words run to the next separator; they expand per execution.
        let mut words = Vec::new();
        while let Some(Token::Word(w)) = self.cur() {
            words.push(w.clone());
            self.pos += 1;
        }

        self.expect_keyword("do")?;
        let body = Box::new(self.parse_list(&["done"])?);
        self.expect_keyword("done")?;
        Ok(Ast::For { var, words, body })
    }

    fn parse_group(&mut self) -> Result<Ast, ParseError> {
        self.pos += 1; // "{"
        let body = self.parse_list(&["}"])?;
        self.expect_keyword("}")?;
        Ok(body)
    }

    fn parse_funcdef(&mut self) -> Result<Ast, ParseError> {
        let word = self.cur_word().unwrap().to_string();
        let name = word.trim_end_matches("()").to_string();
        self.pos += 1;
        self.skip_separators();
        if self.cur().is_none() {
            return Err(ParseError::Incomplete);
        }
        if self.cur_word() != Some("{") {
            return Err(ParseError::Syntax(format!(
                "expected '{{' after '{}()'",
                name
            )));
        }
        let body = Box::new(self.parse_group()?);
        Ok(Ast::FuncDef { name, body })
    }

    fn peek_keyword(&mut self) -> Option<&str> {
        self.skip_separators();
        self.cur_word()
    }
}

fn is_valid_name(word: &str) -> bool {
    let mut chars = word.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_funcdef_word(word: &str) -> bool {
    word.strip_suffix("()").is_some_and(is_valid_name)
}

// ── Evaluator ────────────────────────────────────────────────────────────────

pub fn exec(shell: &mut Shell, ast: &Ast) -> Flow {
    exec_item(shell, ast, false)
}

fn exec_item(shell: &mut Shell, ast: &Ast, background: bool) -> Flow {
    match ast {
        Ast::Pipeline(tokens) => exec_pipeline(shell, tokens, background),

        Ast::List(items) => {
            let mut last = shell.last_status;
            let mut prev_sep = Sep::Seq;
            for (item, sep) in items {
                let should_run = match prev_sep {
                    Sep::And => last == 0,
                    Sep::Or => last != 0,
                    _ => true,
                };
                if should_run {
                    match exec_item(shell, item, *sep == Sep::Background) {
                        Flow::Normal(status) => {
                            last = status;
                            shell.last_status = status;
                            // set -e: a failing untested command exits the
                            // shell (statuses feeding && / || are tested).
                            if status != 0
                                && !matches!(sep, Sep::And | Sep::Or)
                                && shell.condition_depth == 0
                                && shell.options.get("errexit").copied().unwrap_or(false)
                            {
                                std::process::exit(status);
                            }
                        }
                        other => return other,
                    }
                }
                prev_sep = *sep;
            }
            Flow::Normal(last)
        }

        Ast::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            shell.condition_depth += 1;
            let cond_flow = exec(shell, cond);
            shell.condition_depth -= 1;
            match cond_flow {
                Flow::Normal(0) => return exec(shell, then_body),
                Flow::Normal(_) => {}
                other => return other,
            }
            for (elif_cond, elif_body) in elifs {
                shell.condition_depth += 1;
                let flow = exec(shell, elif_cond);
                shell.condition_depth -= 1;
                match flow {
                    Flow::Normal(0) => return exec(shell, elif_body),
                    Flow::Normal(_) => {}
                    other => return other,
                }
            }
            match else_body {
                Some(body) => exec(shell, body),
                None => Flow::Normal(0),
            }
        }

        Ast::Loop { until, cond, body } => {
            let mut last = 0;
            loop {
                shell.condition_depth += 1;
                let cond_flow = exec(shell, cond);
                shell.condition_depth -= 1;
                let status = match cond_flow {
                    Flow::Normal(s) => s,
                    other => return other,
                };
                let enter = if *until { status != 0 } else { status == 0 };
                if !enter {
                    break;
                }
                match exec(shell, body) {
                    Flow::Normal(s) => last = s,
                    Flow::Break(1) => break,
                    Flow::Break(n) => return Flow::Break(n - 1),
                    Flow::Continue(1) => continue,
                    Flow::Continue(n) => return Flow::Continue(n - 1),
                    ret @ Flow::Return(_) => return ret,
                }
            }
            Flow::Normal(last)
        }

        Ast::For { var, words, body } => {
            let fields: Vec<String> = words
                .iter()
                .flat_map(|w| crate::expansion::expand_word(w, shell))
                .collect();
            let mut last = 0;
            for field in fields {
                shell.variables.insert(var.clone(), field);
                match exec(shell, body) {
                    Flow::Normal(s) => last = s,
                    Flow::Break(1) => break,
                    Flow::Break(n) => return Flow::Break(n - 1),
                    Flow::Continue(1) => continue,
                    Flow::Continue(n) => return Flow::Continue(n - 1),
                    ret @ Flow::Return(_) => return ret,
                }
            }
            Flow::Normal(last)
        }

        Ast::FuncDef { name, body } => {
            shell.functions.insert(name.clone(), body.as_ref().clone());
            Flow::Normal(0)
        }
    }
}

/// Execute one simple pipeline, intercepting the control-flow words that
/// must unwind through the AST rather than run as commands.
fn exec_pipeline(shell: &mut Shell, tokens: &[Token], background: bool) -> Flow {
    if let Some(Token::Word(first)) = tokens.first() {
        let count = || -> u32 {
            match tokens.get(1) {
                Some(Token::Word(n)) => n.parse().unwrap_or(1).max(1),
                _ => 1,
            }
        };
        match first.as_str() {
            "break" => return Flow::Break(count()),
            "continue" => return Flow::Continue(count()),
            "return" => {
                let status = match tokens.get(1) {
                    Some(Token::Word(n)) => n.parse().unwrap_or(shell.last_status),
                    _ => shell.last_status,
                };
                return Flow::Return(status);
            }
            _ => {}
        }
    }
    Flow::Normal(shell.eval_tokens(tokens.to_vec(), background))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_str(input: &str) -> Result<Ast, ParseError> {
        parse(tokenize(input))
    }

    #[test]
    fn simple_pipeline_parses() {
        assert!(matches!(parse_str("echo hi"), Ok(Ast::List(_))));
    }

    #[test]
    fn if_requires_fi() {
        assert!(matches!(
            parse_str("if true; then echo hi"),
            Err(ParseError::Incomplete)
        ));
        assert!(parse_str("if true; then echo hi; fi").is_ok());
    }

    #[test]
    fn incomplete_loop_and_group() {
        assert!(matches!(
            parse_str("while true; do echo hi"),
            Err(ParseError::Incomplete)
        ));
        assert!(matches!(
            parse_str("{ echo hi"),
            Err(ParseError::Incomplete)
        ));
        assert!(parse_str("while false; do echo hi; done").is_ok());
    }

    #[test]
    fn stray_terminator_is_syntax_error() {
        assert!(matches!(parse_str("fi"), Err(ParseError::Syntax(_))));
        assert!(matches!(parse_str("done"), Err(ParseError::Syntax(_))));
    }

    #[test]
    fn keywords_as_arguments_are_plain_words() {
        // `echo done` — "done" is not at command position.
        assert!(parse_str("echo done").is_ok());
        assert!(parse_str("echo if then fi").is_ok());
    }

    #[test]
    fn funcdef_parses() {
        let ast = parse_str("greet() { echo hi; }").unwrap();
        let Ast::List(items) = ast else {
            panic!("expected list")
        };
        assert!(matches!(&items[0].0, Ast::FuncDef { name, .. } if name == "greet"));
    }

    #[test]
    fn for_loop_parses() {
        let ast = parse_str("for x in a b c; do echo $x; done").unwrap();
        let Ast::List(items) = ast else {
            panic!("expected list")
        };
        assert!(
            matches!(&items[0].0, Ast::For { var, words, .. } if var == "x" && words.len() == 3)
        );
    }
}
