use std::sync::atomic::{AtomicBool, Ordering};
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use super::job::{Job, JobStatus};

/// Set by the signal handler; drained by Shell::reap_jobs() on the next REPL tick.
static SIGCHLD_PENDING: AtomicBool = AtomicBool::new(false);

pub mod sigchld {
    use super::*;

    /// Register the SIGCHLD handler. Call once at shell startup.
    pub fn install() -> nix::Result<()> { 
        let sa = SigAction::new(
            SigHandler::Handler(handle), 
            SaFlags::SA_RESTART | SaFlags::SA_NOCLDSTOP, 
            SigSet::empty(),
        );
        unsafe { sigaction(Signal::SIGCHLD, &sa)?; }
        Ok(())
    }
    /// The actual signal handler — async-signal-safe, does almost nothing.
    extern "C" fn handle(_sig: libc::c_int) {
        SIGCHLD_PENDING.store(true, Ordering::Relaxed);
    }

    /// Atomically check-and-clear the pending flag.
    pub fn take_pending() -> bool {
        SIGCHLD_PENDING.swap(false, Ordering::AcqRel)
    }
}

/// Reap any finished or stopped children non-blockingly.
/// Called from the REPL loop when sigchld::take_pending() returns true.
pub fn reap(jobs: &mut Vec<Job>) {
    loop {
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Exited(pid, code))      => update_job(jobs, pid, ProcessStatus::Exited(code)),
            Ok(WaitStatus::Signaled(pid, sig, _))  => update_job(jobs, pid, ProcessStatus::Signaled(sig)),
            Ok(WaitStatus::Stopped(pid, _sig))     => update_job(jobs, pid, ProcessStatus::Stopped),
            // now more children to reap
            Ok (WaitStatus::StillAlive) | Ok (WaitStatus::Continued(_)) => break,
            // No Children Exist 
            Err(nix::errno::Errno::ECHILD) => break,
            Err(e) => {
                eprintln!("rsh: waitpid error: {e}");
                break;
            }
            Ok (_) => continue,
        }
    }
    // Prune fully dead jobs
    jobs.retain(|j| !j.is_done());
}

fn update_job(jobs: &mut Vec<Job>, pid: Pid, process_status: ProcessStatus) {
    if let Some(job) = jobs.iter_mut().find(|j| j.processes.iter().any(|p| p.pid == pid)) {
        if let Some(proc) = job.processes.iter_mut().find(|p| p.pid == pid) {
            proc.status = process_status;
        }
        job.update_status();
    }
}
