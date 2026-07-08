//! Word expansion: tilde, parameter (`$VAR`, `${VAR}`, `$?`, `$$`, `$0`-`$9`,
//! `$#`), field splitting, glob expansion, and quote removal.
//!
//! The lexer keeps quote characters in `Word` tokens; this pass runs on each
//! parsed `Command` (between parse and dispatch) and is responsible for
//! removing them. Semantics follow POSIX/bash:
//!
//! - single quotes: everything literal
//! - double quotes: `$` expands, no field splitting, no globbing
//! - unquoted: `$` expands, results are field-split on whitespace, and
//!   fields containing unquoted `*`/`?`/`[` are glob-expanded (a pattern
//!   with no matches is kept literally, like bash without `nullglob`)

use crate::ast::{Command, Pipeline, Redirect};
use crate::shell::Shell;
use std::iter::Peekable;
use std::str::Chars;

/// Expand every word in every command of the pipeline. A word may expand to
/// zero fields (`$UNSET`) or several (`$X` where `X='a b'`). Redirect
/// targets are expanded without field splitting or globbing.
pub fn expand_pipeline(pipeline: Pipeline, shell: &Shell) -> Pipeline {
    let commands = pipeline
        .commands
        .into_iter()
        .map(|cmd| {
            let argv = cmd
                .argv
                .iter()
                .flat_map(|w| expand_word(w, shell))
                .collect();
            let stdin = expand_redirect(cmd.stdin, shell);
            let stdout = expand_redirect(cmd.stdout, shell);
            Command::new(argv, stdin, stdout)
        })
        .collect();
    Pipeline::new(commands, pipeline.background)
}

fn expand_redirect(redirect: Redirect, shell: &Shell) -> Redirect {
    match redirect {
        Redirect::File(path) => Redirect::File(expand_word_no_split(&path, shell)),
        Redirect::Append(path) => Redirect::Append(expand_word_no_split(&path, shell)),
        other => other,
    }
}

/// Full expansion of a single word into zero or more fields.
pub fn expand_word(word: &str, shell: &Shell) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut field_globs: Vec<bool> = Vec::new();
    let mut current = String::new();
    // True once `current` is a real field, even if empty (`""` must survive
    // as an empty argument, while a fully-empty `$UNSET` must disappear).
    let mut has_field = false;
    let mut glob_pending = false;

    let close_field = |current: &mut String,
                       has_field: &mut bool,
                       glob_pending: &mut bool,
                       fields: &mut Vec<String>,
                       field_globs: &mut Vec<bool>| {
        fields.push(std::mem::take(current));
        field_globs.push(std::mem::take(glob_pending));
        *has_field = false;
    };

    let mut chars = word.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                has_field = true;
                for q in chars.by_ref() {
                    if q == '\'' {
                        break;
                    }
                    current.push(q);
                }
            }
            '"' => {
                has_field = true;
                while let Some(q) = chars.next() {
                    match q {
                        '"' => break,
                        '$' => current.push_str(&expand_dollar(&mut chars, shell)),
                        _ => current.push(q),
                    }
                }
            }
            '$' => {
                let val = expand_dollar(&mut chars, shell);
                // Unquoted expansion: split into fields on whitespace.
                let starts_ws = val.starts_with(char::is_whitespace);
                let ends_ws = val.ends_with(char::is_whitespace);
                let parts: Vec<&str> = val.split_whitespace().collect();

                if starts_ws && (has_field || !current.is_empty()) {
                    close_field(
                        &mut current,
                        &mut has_field,
                        &mut glob_pending,
                        &mut fields,
                        &mut field_globs,
                    );
                }
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        close_field(
                            &mut current,
                            &mut has_field,
                            &mut glob_pending,
                            &mut fields,
                            &mut field_globs,
                        );
                    }
                    if part.contains(['*', '?', '[']) {
                        glob_pending = true;
                    }
                    current.push_str(part);
                    has_field = true;
                }
                if ends_ws && !parts.is_empty() {
                    close_field(
                        &mut current,
                        &mut has_field,
                        &mut glob_pending,
                        &mut fields,
                        &mut field_globs,
                    );
                }
            }
            '~' if current.is_empty() && !has_field && fields.is_empty() => {
                // Tilde only expands at the very start of an unquoted word,
                // and only for bare `~` or `~/...`.
                if matches!(chars.peek(), None | Some('/')) {
                    let home = shell
                        .env
                        .get("HOME")
                        .cloned()
                        .unwrap_or_else(|| "~".to_string());
                    current.push_str(&home);
                } else {
                    current.push('~');
                }
                has_field = true;
            }
            '*' | '?' | '[' => {
                glob_pending = true;
                current.push(c);
                has_field = true;
            }
            _ => {
                current.push(c);
                has_field = true;
            }
        }
    }
    if has_field {
        fields.push(current);
        field_globs.push(glob_pending);
    }

    // Glob pass: expand fields that contained unquoted pattern characters.
    let options = glob::MatchOptions {
        require_literal_leading_dot: true,
        ..Default::default()
    };
    let mut out = Vec::new();
    for (field, had_glob) in fields.into_iter().zip(field_globs) {
        if !had_glob {
            out.push(field);
            continue;
        }
        match glob::glob_with(&field, options) {
            Ok(paths) => {
                let mut matches: Vec<String> = paths
                    .filter_map(Result::ok)
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                if matches.is_empty() {
                    out.push(field); // no match: keep the pattern literally
                } else {
                    matches.sort();
                    out.append(&mut matches);
                }
            }
            Err(_) => out.push(field),
        }
    }
    out
}

/// Expansion without field splitting or globbing — for redirect targets and
/// assignment values, where the result must stay a single string.
pub fn expand_word_no_split(word: &str, shell: &Shell) -> String {
    let mut result = String::new();
    let mut chars = word.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                for q in chars.by_ref() {
                    if q == '\'' {
                        break;
                    }
                    result.push(q);
                }
            }
            '"' => {
                while let Some(q) = chars.next() {
                    match q {
                        '"' => break,
                        '$' => result.push_str(&expand_dollar(&mut chars, shell)),
                        _ => result.push(q),
                    }
                }
            }
            '$' => result.push_str(&expand_dollar(&mut chars, shell)),
            '~' if result.is_empty() => {
                if matches!(chars.peek(), None | Some('/')) {
                    let home = shell
                        .env
                        .get("HOME")
                        .cloned()
                        .unwrap_or_else(|| "~".to_string());
                    result.push_str(&home);
                } else {
                    result.push('~');
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Consume a parameter reference after a `$` and return its value.
/// Supports `$NAME`, `${NAME}`, `$?`, `$$`, `$#`, and `$0`-`$9`.
/// A `$` followed by nothing expandable stays a literal `$`.
fn expand_dollar(chars: &mut Peekable<Chars>, shell: &Shell) -> String {
    match chars.peek() {
        Some('?') => {
            chars.next();
            shell.last_status.to_string()
        }
        Some('$') => {
            chars.next();
            std::process::id().to_string()
        }
        Some('#') => {
            chars.next();
            let mut n = 0;
            while shell.variables.contains_key(&(n + 1).to_string()) {
                n += 1;
            }
            n.to_string()
        }
        Some('{') => {
            chars.next();
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            lookup(&name, shell)
        }
        Some(c) if c.is_ascii_digit() => {
            let name = chars.next().unwrap().to_string();
            lookup(&name, shell)
        }
        Some(c) if c.is_alphabetic() || *c == '_' => {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            lookup(&name, shell)
        }
        _ => "$".to_string(),
    }
}

fn lookup(name: &str, shell: &Shell) -> String {
    if name == "0" {
        return "rsh".to_string();
    }
    shell
        .variables
        .get(name)
        .or_else(|| shell.env.get(name))
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_with(vars: &[(&str, &str)]) -> Shell {
        let mut shell = Shell::new();
        for (k, v) in vars {
            shell.variables.insert(k.to_string(), v.to_string());
        }
        shell
    }

    #[test]
    fn plain_word_passes_through() {
        let shell = shell_with(&[]);
        assert_eq!(expand_word("hello", &shell), vec!["hello"]);
    }

    #[test]
    fn variable_expands() {
        let shell = shell_with(&[("X", "value")]);
        assert_eq!(expand_word("$X", &shell), vec!["value"]);
        assert_eq!(expand_word("${X}", &shell), vec!["value"]);
        assert_eq!(expand_word("pre-${X}-post", &shell), vec!["pre-value-post"]);
    }

    #[test]
    fn unset_variable_vanishes() {
        let shell = shell_with(&[]);
        assert!(expand_word("$UNSET_VAR_XYZ", &shell).is_empty());
    }

    #[test]
    fn unset_variable_in_quotes_is_empty_field() {
        let shell = shell_with(&[]);
        assert_eq!(expand_word("\"$UNSET_VAR_XYZ\"", &shell), vec![""]);
    }

    #[test]
    fn unquoted_expansion_field_splits() {
        let shell = shell_with(&[("X", "a b  c")]);
        assert_eq!(expand_word("$X", &shell), vec!["a", "b", "c"]);
    }

    #[test]
    fn double_quoted_expansion_does_not_split() {
        let shell = shell_with(&[("X", "a b")]);
        assert_eq!(expand_word("\"$X\"", &shell), vec!["a b"]);
    }

    #[test]
    fn single_quotes_are_literal() {
        let shell = shell_with(&[("X", "value")]);
        assert_eq!(expand_word("'$X'", &shell), vec!["$X"]);
    }

    #[test]
    fn last_status_expands() {
        let mut shell = shell_with(&[]);
        shell.last_status = 42;
        assert_eq!(expand_word("$?", &shell), vec!["42"]);
    }

    #[test]
    fn lone_dollar_is_literal() {
        let shell = shell_with(&[]);
        assert_eq!(expand_word("$", &shell), vec!["$"]);
        assert_eq!(expand_word("a$", &shell), vec!["a$"]);
    }

    #[test]
    fn tilde_expands_to_home() {
        let mut shell = shell_with(&[]);
        shell.env.insert("HOME".into(), "/home/test".into());
        assert_eq!(expand_word("~", &shell), vec!["/home/test"]);
        assert_eq!(expand_word("~/sub", &shell), vec!["/home/test/sub"]);
        // Not at word start, or quoted: literal.
        assert_eq!(expand_word("a~b", &shell), vec!["a~b"]);
        assert_eq!(expand_word("'~'", &shell), vec!["~"]);
    }

    #[test]
    fn quote_removal_glues_runs() {
        let shell = shell_with(&[]);
        assert_eq!(expand_word("ll='ls -la'", &shell), vec!["ll=ls -la"]);
    }

    #[test]
    fn no_split_keeps_one_string() {
        let shell = shell_with(&[("X", "a b")]);
        assert_eq!(expand_word_no_split("$X.txt", &shell), "a b.txt");
    }

    #[test]
    fn positional_params_expand() {
        let shell = shell_with(&[("1", "first"), ("2", "second")]);
        assert_eq!(expand_word("$1", &shell), vec!["first"]);
        assert_eq!(expand_word("$2", &shell), vec!["second"]);
        assert_eq!(expand_word("$#", &shell), vec!["2"]);
    }
}
