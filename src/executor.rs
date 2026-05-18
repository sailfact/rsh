use nix::unistd::{fork, ForkResult, pipe, close, setpgid, getpid, getpgrp};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::os::unix::io::{RawFd, IntoRawFd};

use crate::ast::pipeline::Pipeline;
use crate::ast::command::Command;
use crate::ast::redirect::Redirect;
use crate::builtins;
use crate::jobs::job::Job;
use crate::jobs::process::{Process, ProcessStatus};
use crate::shell::Shell;
use crate::external;

pub fn execute(shell: &mut Shell, pipeline: Pipeline) -> i32 {
    let n = pipeline.commands.len();
    if n == 0 {
        return 0;
    }

    // Create n-1 pipes, one between each adjacent pair of commands.
    // Convert OwnedFd -> RawFd immediately so we manage lifetime manually.
    let mut pipes: Vec<(RawFd, RawFd)> = Vec::new();
    for _ in 0..n.saturating_sub(1) {
        let (r, w) = pipe().expect("pipe() failed");
        pipes.push((r.into_raw_fd(), w.into_raw_fd()));
    }

    let mut pgid      = Pid::from_raw(0);
    let mut processes: Vec<Process> = Vec::new();

    for (i, cmd) in pipeline.commands.iter().enumerate() {
        let stdin_fd  = if i == 0     { None } else { Some(pipes[i - 1].0) };
        let stdout_fd = if i == n - 1 { None } else { Some(pipes[i].1) };

        match unsafe { fork().expect("fork() failed") } {

            ForkResult::Child => {
                // ── Child process ─────────────────────────────────────────

                let child_pid  = getpid();
                let child_pgid = if pgid.as_raw() == 0 { child_pid } else { pgid };
                setpgid(child_pid, child_pgid).ok();

                // Wire up pipe ends
                if let Some(fd) = stdin_fd {
                    unsafe { libc::dup2(fd, 0); }
                }
                if let Some(fd) = stdout_fd {
                    unsafe { libc::dup2(fd, 1); }
                }

                // Apply explicit file redirections
                apply_redirects(cmd);

                // Close all pipe fds — we only need the dup'd copies
                for (r, w) in &pipes {
                    close(*r).ok();
                    close(*w).ok();
                }

                // ── Dispatch ──────────────────────────────────────────────
                //
                // rshell builtins (cd, alias, export etc.) are NEVER
                // reached here — shell.rs handles them before calling
                // executor::execute(). Only coreutils and external reach
                // this point.

                let name = cmd.argv[0].as_str();

                if builtins::is_uutils_builtin(name) {
                    let status = builtins::exec_uutils_builtin(cmd);
                    std::process::exit(status);
                } else {
                    external::exec(cmd); // never returns
                }
            }

            ForkResult::Parent { child } => {
                // ── Parent process ────────────────────────────────────────

                if pgid.as_raw() == 0 {
                    pgid = child;
                    setpgid(child, child).ok();
                } else {
                    setpgid(child, pgid).ok();
                }

                processes.push(Process {
                    pid:    child,
                    argv:   cmd.argv.clone(),
                    status: ProcessStatus::Running,
                });
            }
        }
    }

    // Parent closes all pipe fds
    for (r, w) in &pipes {
        close(*r).ok();
        close(*w).ok();
    }

    // Register the job
    let id  = shell.jobs.len() + 1;
    let job = Job::new(id, pgid, processes);
    shell.jobs.push(job);
    let job_idx = shell.jobs.len() - 1;

    if pipeline.background {
        eprintln!("[{}] {}", shell.jobs[job_idx].id, pgid);
        0
    } else {
        wait_foreground(job_idx, shell)
    }
}

fn wait_foreground(job_idx: usize, shell: &mut Shell) -> i32 {
    let pgid = shell.jobs[job_idx].pgid;

    unsafe { libc::tcsetpgrp(0, pgid.as_raw()); }

    let mut last_status = 0;

    loop {
        match waitpid(
            Pid::from_raw(-pgid.as_raw()),
            Some(WaitPidFlag::WUNTRACED),
        ) {
            Ok(WaitStatus::Exited(pid, code)) => {
                shell.update_process(pid, ProcessStatus::Exited(code));
                last_status = code;
            }
            Ok(WaitStatus::Stopped(pid, sig)) => {
                shell.update_process(pid, ProcessStatus::Stopped(sig));
                eprintln!("\n[{}] Stopped", shell.jobs[job_idx].id);
                break;
            }
            Ok(WaitStatus::Signaled(pid, sig, _)) => {
                shell.update_process(pid, ProcessStatus::Signaled(sig));
                last_status = 128 + sig as i32;
            }
            Ok(_)  => continue,
            Err(_) => break,
        }

        if shell.jobs[job_idx].is_done() {
            break;
        }
    }

    unsafe { libc::tcsetpgrp(0, getpgrp().as_raw()); }
    shell.jobs.retain(|j| !j.is_done());

    last_status
}

fn apply_redirects(cmd: &Command) {
    use std::fs::OpenOptions;
    use std::os::unix::io::IntoRawFd;

    match &cmd.stdin {
        Redirect::File(path) => {
            let fd = OpenOptions::new()
                .read(true)
                .open(path)
                .expect("failed to open stdin redirect")
                .into_raw_fd();
            unsafe { libc::dup2(fd, 0); }
        }
        _ => {}
    }

    match &cmd.stdout {
        Redirect::File(path) => {
            let fd = OpenOptions::new()
                .write(true).create(true).truncate(true)
                .open(path)
                .expect("failed to open stdout redirect")
                .into_raw_fd();
            unsafe { libc::dup2(fd, 1); }
        }
        Redirect::Append(path) => {
            let fd = OpenOptions::new()
                .write(true).create(true).append(true)
                .open(path)
                .expect("failed to open stdout append redirect")
                .into_raw_fd();
            unsafe { libc::dup2(fd, 1); }
        }
        _ => {}
    }
}
