use super::Builtin;
use crate::shell::Shell;

pub struct Wait;

impl Builtin for Wait {
    fn name(&self) -> &'static str {
        "wait"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            // wait for all jobs
            let mut last = 0;
            // collect pgids first to avoid borrow issues
            let pgids: Vec<_> = shell.jobs.iter().map(|j| j.pgid).collect();
            for pgid in pgids {
                last = wait_pgid(shell, pgid);
            }
            shell.jobs.retain(|j| !j.is_done());
            return last;
        }

        let mut status = 0;
        for spec in &args[1..] {
            if spec.starts_with('%') {
                let pgid = match shell.find_job_mut(Some(spec.as_str())) {
                    Some(j) => j.pgid,
                    None => {
                        eprintln!("wait: {}: no such job", spec);
                        status = 127;
                        continue;
                    }
                };
                status = wait_pgid(shell, pgid);
            } else {
                match spec.parse::<i32>() {
                    Ok(pid) => {
                        status = wait_pid(nix::unistd::Pid::from_raw(pid));
                    }
                    Err(_) => {
                        eprintln!("wait: {}: invalid pid", spec);
                        status = 1;
                    }
                }
            }
        }
        shell.jobs.retain(|j| !j.is_done());
        status
    }
}

fn wait_pid(pid: nix::unistd::Pid) -> i32 {
    use nix::sys::wait::{WaitStatus, waitpid};
    loop {
        match waitpid(pid, None) {
            Ok(WaitStatus::Exited(_, code)) => return code,
            Ok(WaitStatus::Signaled(_, sig, _)) => return 128 + sig as i32,
            Ok(_) => continue,
            Err(_) => return 1,
        }
    }
}

fn wait_pgid(shell: &mut Shell, pgid: nix::unistd::Pid) -> i32 {
    use crate::jobs::process::ProcessStatus;
    use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

    let mut last = 0;
    loop {
        match waitpid(
            nix::unistd::Pid::from_raw(-pgid.as_raw()),
            Some(WaitPidFlag::WNOHANG),
        ) {
            Ok(WaitStatus::Exited(pid, code)) => {
                shell.update_process(pid, ProcessStatus::Exited(code));
                last = code;
            }
            Ok(WaitStatus::Signaled(pid, sig, _)) => {
                shell.update_process(pid, ProcessStatus::Signaled(sig));
                last = 128 + sig as i32;
            }
            Ok(WaitStatus::StillAlive) => {
                // use blocking wait when WNOHANG says still alive
                match waitpid(nix::unistd::Pid::from_raw(-pgid.as_raw()), None) {
                    Ok(WaitStatus::Exited(pid, code)) => {
                        shell.update_process(pid, ProcessStatus::Exited(code));
                        last = code;
                    }
                    Ok(WaitStatus::Signaled(pid, sig, _)) => {
                        shell.update_process(pid, ProcessStatus::Signaled(sig));
                        last = 128 + sig as i32;
                    }
                    _ => {}
                }
            }
            _ => break,
        }

        let still_running = shell
            .jobs
            .iter()
            .filter(|j| j.pgid == pgid)
            .any(|j| !j.is_done());
        if !still_running {
            break;
        }
    }
    last
}
