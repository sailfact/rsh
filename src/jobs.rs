// Jobs: Process Group, Statuese, and SIGCHLD handling
use std::sync::atomic::{AtomicBool, Ordering};
use nix::sys::signal::Signal;
use nix::unistd::Pid;

// Types
pub struct Job {
    pub id: usize,
    pub pgid: pid, 
    pub processes: Vec<Process>,
    pub status: JobStatus,
    pub command_line: String,
}

pub struct Process {
    pub pid: Pid,
    pub argv: Vec<String>,
    pub status: ProcessStatus,
}

pub enum JobStatus {
    Running,
    Stopped,
    Done(i32),
}

pub enum ProcessStatus {
    Running,
    Exited(i32),
    Signaled(Signal),
    Stopped,
}

// Job impl
impl Job {
    pub fn new(id: usize, pgid: Pid, processes: Vec<Process>, command_line: String) -> Self { 
        
    }

    /// Recompute aggregate status from the underlying processes.
    pub fn update_status(&mut self) { ... }

    /// Send a signal to the entire process group.
    pub fn send_signal(&self, sig: Signal) -> nix::Result<()> { ... }

    pub fn is_done(&self) -> bool { ... }
}

// ---------- SIGCHLD handling ----------

/// Set by the signal handler; drained by Shell::reap_jobs() on the next REPL tick.
static SIGCHLD_PENDING: AtomicBool = AtomicBool::new(false);

pub mod sigchld {
    use super::*;

    /// Register the SIGCHLD handler. Call once at shell startup.
    pub fn install() -> nix::Result<()> { ... }

    /// The actual signal handler — async-signal-safe, does almost nothing.
    extern "C" fn handle(_sig: libc::c_int) { ... }

    /// Atomically check-and-clear the pending flag.
    pub fn take_pending() -> bool { ... }
}

/// Reap any finished or stopped children non-blockingly.
/// Called from the REPL loop when sigchld::take_pending() returns true.
pub fn reap(jobs: &mut Vec<Job>) { ... }
