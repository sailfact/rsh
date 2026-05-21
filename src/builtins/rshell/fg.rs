use super::Builtin;
use crate::jobs::job::JobStatus;
use crate::shell::Shell;
use nix::sys::signal::Signal;

pub struct Fg;

impl Builtin for Fg {
    fn name(&self) -> &'static str {
        "fg"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let spec = args.get(1).map(String::as_str);

        let (pgid, argv) = match shell.find_job_mut(spec) {
            None => {
                eprintln!("fg: no such job");
                return 1;
            }
            Some(job) => {
                job.status = JobStatus::Running;
                let pgid = job.pgid;
                let argv = job.argv_string();
                (pgid, argv)
            }
        };

        eprintln!("{}", argv);

        // give terminal control to the job
        unsafe {
            libc::tcsetpgrp(shell.tty_fd, pgid.as_raw());
        }

        // resume it
        if let Err(e) =
            nix::sys::signal::kill(nix::unistd::Pid::from_raw(-pgid.as_raw()), Signal::SIGCONT)
        {
            eprintln!("fg: SIGCONT failed: {}", e);
        }

        // wait for it to stop or finish
        let mut last_status = 0;
        loop {
            use crate::jobs::process::ProcessStatus;
            use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};

            match waitpid(
                nix::unistd::Pid::from_raw(-pgid.as_raw()),
                Some(WaitPidFlag::WUNTRACED),
            ) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    shell.update_process(pid, ProcessStatus::Exited(code));
                    last_status = code;
                }
                Ok(WaitStatus::Stopped(pid, sig)) => {
                    shell.update_process(pid, ProcessStatus::Stopped(sig));
                    eprintln!("\n[?] Stopped");
                    break;
                }
                Ok(WaitStatus::Signaled(pid, sig, _)) => {
                    shell.update_process(pid, ProcessStatus::Signaled(sig));
                    last_status = 128 + sig as i32;
                }
                Ok(_) => continue,
                Err(_) => break,
            }

            // check if any process in this pgid is still running
            let still_running = shell
                .jobs
                .iter()
                .filter(|j| j.pgid == pgid)
                .any(|j| !j.is_done());
            if !still_running {
                break;
            }
        }

        // return terminal to shell
        unsafe {
            libc::tcsetpgrp(shell.tty_fd, shell.shell_pgid.as_raw());
        }
        shell.jobs.retain(|j| !j.is_done());

        last_status
    }
}
