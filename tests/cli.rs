//! End-to-end tests driving the rsh binary in non-interactive mode.
//!
//! Each test spawns `rsh -c '...'` (or feeds a script) and asserts on
//! stdout/stderr/exit status, so the whole lex → parse → execute path is
//! exercised, including forked children and pipe wiring.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const RSH: &str = env!("CARGO_BIN_EXE_rsh");

fn rsh_c(cmd: &str) -> Output {
    Command::new(RSH)
        .args(["-c", cmd])
        .output()
        .expect("failed to spawn rsh")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rsh-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

// ── Single commands & exit status ────────────────────────────────────────────

#[test]
fn echo_builtin_prints() {
    let out = rsh_c("echo hello world");
    assert_eq!(stdout(&out), "hello world\n");
    assert!(out.status.success());
}

#[test]
fn true_and_false_statuses() {
    assert_eq!(rsh_c("true").status.code(), Some(0));
    assert_eq!(rsh_c("false").status.code(), Some(1));
}

#[test]
fn exit_builtin_sets_status() {
    assert_eq!(rsh_c("exit 42").status.code(), Some(42));
}

#[test]
fn shell_exits_with_last_command_status() {
    assert_eq!(rsh_c("echo ok; false").status.code(), Some(1));
    assert_eq!(rsh_c("false; echo ok").status.code(), Some(0));
}

#[test]
fn command_not_found_is_127() {
    let out = rsh_c("definitely-not-a-command-xyz");
    assert_eq!(out.status.code(), Some(127));
}

#[test]
fn external_command_runs() {
    let out = rsh_c("/bin/echo external");
    assert_eq!(stdout(&out), "external\n");
}

// ── Semicolons & comments ────────────────────────────────────────────────────

#[test]
fn semicolons_run_sequentially() {
    let out = rsh_c("echo a; echo b; echo c");
    assert_eq!(stdout(&out), "a\nb\nc\n");
}

#[test]
fn trailing_semicolon_is_ignored() {
    let out = rsh_c("echo a;");
    assert_eq!(stdout(&out), "a\n");
    assert!(out.status.success());
}

#[test]
fn comment_lines_are_skipped() {
    let out = rsh_c("# just a comment\necho visible");
    assert_eq!(stdout(&out), "visible\n");
    assert!(out.status.success());
}

// ── Pipelines ────────────────────────────────────────────────────────────────

#[test]
fn two_stage_pipeline() {
    let out = rsh_c("echo one two three | wc -w");
    assert_eq!(stdout(&out).trim(), "3");
    assert!(out.status.success());
}

#[test]
fn three_stage_pipeline() {
    let out = rsh_c("printf 'b\\na\\nb\\n' | sort | uniq");
    assert_eq!(stdout(&out), "a\nb\n");
}

#[test]
fn pipeline_status_is_last_command() {
    // `false | echo ok` — last command succeeds, so the pipeline does too.
    assert_eq!(rsh_c("false | echo ok").status.code(), Some(0));
}

// ── Redirection ──────────────────────────────────────────────────────────────

#[test]
fn redirect_out_creates_file() {
    let dir = tempdir("redirect-out");
    let file = dir.join("out.txt");
    let out = rsh_c(&format!("echo written > {}", file.display()));
    assert!(out.status.success());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "written\n");
}

#[test]
fn redirect_append_adds_to_file() {
    let dir = tempdir("redirect-append");
    let file = dir.join("out.txt");
    let script = format!("echo first > {p}; echo second >> {p}", p = file.display());
    assert!(rsh_c(&script).status.success());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "first\nsecond\n");
}

#[test]
fn redirect_in_feeds_stdin() {
    let dir = tempdir("redirect-in");
    let file = dir.join("in.txt");
    std::fs::write(&file, "from-file\n").unwrap();
    let out = rsh_c(&format!("cat < {}", file.display()));
    assert_eq!(stdout(&out), "from-file\n");
}

#[test]
fn missing_input_redirect_fails_without_crashing() {
    let out = rsh_c("cat < /definitely/not/a/file");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("rsh:"), "expected shell error, got: {err}");
    // A panic would print a backtrace marker instead of a one-line error.
    assert!(!err.contains("panicked"), "child panicked: {err}");
}

#[test]
fn builtin_redirect_restores_shell_stdout() {
    let dir = tempdir("builtin-restore");
    let file = dir.join("out.txt");
    let out = rsh_c(&format!("pwd > {}; echo visible", file.display()));
    assert_eq!(stdout(&out), "visible\n");
    assert!(!std::fs::read_to_string(&file).unwrap().is_empty());
}

// ── Script & stdin modes ─────────────────────────────────────────────────────

#[test]
fn script_file_runs_to_completion() {
    let dir = tempdir("script");
    let script = dir.join("test.rsh");
    std::fs::write(&script, "# a script\necho one\necho two; exit 7\n").unwrap();
    let out = Command::new(RSH)
        .arg(&script)
        .output()
        .expect("failed to spawn rsh");
    assert_eq!(stdout(&out), "one\ntwo\n");
    assert_eq!(out.status.code(), Some(7));
}

#[test]
fn missing_script_is_127() {
    let out = Command::new(RSH)
        .arg("/definitely/not/a/script.rsh")
        .output()
        .expect("failed to spawn rsh");
    assert_eq!(out.status.code(), Some(127));
}

#[test]
fn invalid_option_is_2() {
    let out = Command::new(RSH)
        .arg("--bogus")
        .output()
        .expect("failed to spawn rsh");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn dash_c_without_argument_is_2() {
    let out = Command::new(RSH)
        .arg("-c")
        .output()
        .expect("failed to spawn rsh");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn piped_stdin_is_evaluated() {
    let mut child = Command::new(RSH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn rsh");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"echo from-stdin\nexit 3\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(stdout(&out), "from-stdin\n");
    assert_eq!(out.status.code(), Some(3));
}

// ── Background jobs ──────────────────────────────────────────────────────────

#[test]
fn background_job_does_not_block() {
    // Null stdio: the orphaned sleep child would otherwise hold the
    // capture pipes open and stall an .output() call until it exits.
    let start = std::time::Instant::now();
    let status = Command::new(RSH)
        .args(["-c", "sleep 2 &"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to spawn rsh");
    assert!(status.success());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "shell blocked on background job"
    );
}

// ── Builtins that mutate shell state ─────────────────────────────────────────

#[test]
fn cd_changes_directory_for_later_commands() {
    let out = rsh_c("cd /; pwd");
    assert_eq!(stdout(&out), "/\n");
}

#[test]
fn export_is_visible_to_children() {
    let out = rsh_c("export RSH_TEST_VAR=hello; printenv RSH_TEST_VAR");
    assert_eq!(stdout(&out), "hello\n");
}
