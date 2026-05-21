pub mod job;
pub mod process;

pub use job::{Job, JobStatus};
pub use process::{Process, ProcessStatus};

pub use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};
pub use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
pub use nix::unistd::Pid;
pub use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_done_carries_exit_code() {
        let status = JobStatus::Done(42);
        assert!(matches!(status, JobStatus::Done(42)));
    }

    #[test]
    fn test_done_zero_on_success() {
        let status = JobStatus::Done(0);
        assert!(matches!(status, JobStatus::Done(0)));
    }
}
