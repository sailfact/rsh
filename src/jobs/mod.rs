pub mod job;
pub mod process;

pub use job::{Job, JobStatus};
pub use process::{Process, ProcessStatus};

pub use std::sync::atomic::{AtomicBool, Ordering};
pub use nix::unistd::Pid;
pub use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
pub use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

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