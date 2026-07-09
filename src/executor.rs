use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use nix::unistd::{ForkResult, close, fork, getpgrp, getpid, pipe, setpgid};
use std::os::unix::io::{IntoRawFd, RawFd};

use crate::ast::command::Command;
use crate::ast::pipeline::Pipeline;
use crate::ast::redirect::Redirect;
use crate::builtins;
use crate::external;
use crate::jobs::job::Job;
use crate::jobs::process::{Process, ProcessStatus};
use crate::shell::Shell;
use crate::signals;

pub fn execute(shell: &mut Shell, pipeline: Pipeline) -> i32 {
    let n = pipeline.commands.len();
    if n == 0 {
        return 0;
    }

    // Create n-1 pipes, one between each adjacent pair of commands.
    // Convert OwnedFd -> RawFd immediately so we manage lifetime manually.
    let mut pipes: Vec<(RawFd, RawFd)> = Vec::new();
    for _ in 0..n.saturating_sub(1) {
        match pipe() {
            Ok((r, w)) => pipes.push((r.into_raw_fd(), w.into_raw_fd())),
            Err(e) => {
                eprintln!("rsh: pipe: {}", e);
                close_pipes(&pipes);
                return 1;
            }
        }
    }

    // Job control (process groups + terminal handoff) only applies to
    // interactive shells; non-interactive children stay in the shell's own
    // process group, like `sh -c`.
    let interactive = shell.is_interactive;
    let mut pgid = Pid::from_raw(0);
    let mut processes: Vec<Process> = Vec::new();
    let mut spawn_failed = false;

    for (i, cmd) in pipeline.commands.iter().enumerate() {
        let stdin_fd = if i == 0 { None } else { Some(pipes[i - 1].0) };
        let stdout_fd = if i == n - 1 { None } else { Some(pipes[i].1) };

        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                // ── Child process ─────────────────────────────────────────

                // Undo the shell's ignored signals before running anything.
                signals::restore_default_handlers();

                if interactive {
                    let child_pid = getpid();
                    let child_pgid = if pgid.as_raw() == 0 { child_pid } else { pgid };
                    setpgid(child_pid, child_pgid).ok();
                }

                // Wire up pipe ends
                if let Some(fd) = stdin_fd {
                    unsafe {
                        libc::dup2(fd, 0);
                    }
                }
                if let Some(fd) = stdout_fd {
                    unsafe {
                        libc::dup2(fd, 1);
                    }
                }

                // Apply explicit file redirections
                if let Err(msg) = apply_redirects(cmd) {
                    eprintln!("rsh: {}", msg);
                    std::process::exit(1);
                }

                // Close all pipe fds — we only need the dup'd copies
                close_pipes(&pipes);

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

            Ok(ForkResult::Parent { child }) => {
                // ── Parent process ────────────────────────────────────────

                if pgid.as_raw() == 0 {
                    pgid = child;
                }
                if interactive {
                    setpgid(child, pgid).ok();
                }

                processes.push(Process {
                    pid: child,
                    argv: cmd.argv.clone(),
                    status: ProcessStatus::Running,
                });
            }

            Err(e) => {
                eprintln!("rsh: fork: {}", e);
                spawn_failed = true;
                break;
            }
        }
    }

    // Parent closes all pipe fds
    close_pipes(&pipes);

    if processes.is_empty() {
        return 1;
    }

    // Register the job
    let id = shell.next_job_id();
    let job = Job::new(id, pgid, processes);
    shell.jobs.push(job);
    let job_idx = shell.jobs.len() - 1;

    let status = if pipeline.background {
        eprintln!("[{}] {}", shell.jobs[job_idx].id, pgid);
        0
    } else {
        wait_foreground(job_idx, shell)
    };

    if spawn_failed { 1 } else { status }
}

fn wait_foreground(job_idx: usize, shell: &mut Shell) -> i32 {
    let pgid = shell.jobs[job_idx].pgid;
    let interactive = shell.is_interactive;

    if interactive {
        unsafe {
            libc::tcsetpgrp(0, pgid.as_raw());
        }
    }

    // A pipeline's exit status is the status of its last command.
    let last_pid = shell.jobs[job_idx].processes.last().map(|p| p.pid);
    let mut last_status = 0;

    loop {
        // Wait on any child: statuses are recorded per-pid, so children of
        // other (background) jobs reaped here are not lost.
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Exited(pid, code)) => {
                shell.update_process(pid, ProcessStatus::Exited(code));
                if Some(pid) == last_pid {
                    last_status = code;
                }
            }
            Ok(WaitStatus::Signaled(pid, sig, _)) => {
                shell.update_process(pid, ProcessStatus::Signaled(sig));
                if Some(pid) == last_pid {
                    last_status = 128 + sig as i32;
                }
            }
            Ok(WaitStatus::Stopped(pid, sig)) => {
                shell.update_process(pid, ProcessStatus::Stopped(sig));
                let in_job = shell.jobs[job_idx].processes.iter().any(|p| p.pid == pid);
                if in_job {
                    eprintln!("\n[{}] Stopped", shell.jobs[job_idx].id);
                    break;
                }
            }
            Ok(_) => continue,
            Err(_) => break, // ECHILD — nothing left to wait for
        }

        if shell.jobs[job_idx].is_done() {
            break;
        }
    }

    if interactive {
        unsafe {
            libc::tcsetpgrp(0, getpgrp().as_raw());
        }
    }
    shell.jobs.retain(|j| !j.is_done());

    last_status
}

fn close_pipes(pipes: &[(RawFd, RawFd)]) {
    for (r, w) in pipes {
        close(*r).ok();
        close(*w).ok();
    }
}

fn apply_redirects(cmd: &Command) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::os::unix::io::IntoRawFd;

    if let Redirect::File(path) = &cmd.stdin {
        let fd = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| format!("{}: {}", path, e))?
            .into_raw_fd();
        unsafe {
            libc::dup2(fd, 0);
        }
    }

    match &cmd.stdout {
        Redirect::File(path) => {
            let fd = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .map_err(|e| format!("{}: {}", path, e))?
                .into_raw_fd();
            unsafe {
                libc::dup2(fd, 1);
            }
        }
        Redirect::Append(path) => {
            let fd = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| format!("{}: {}", path, e))?
                .into_raw_fd();
            unsafe {
                libc::dup2(fd, 1);
            }
        }
        _ => {}
    }

    Ok(())
}
