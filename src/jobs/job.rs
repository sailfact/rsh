use super::{Process, ProcessStatus, Signal, Pid};

pub struct Job {
    pub id: usize,
    pub pgid: Pid, 
    pub processes: Vec<Process>,
    pub status: JobStatus,
    pub command_line: String,
}
pub enum JobStatus {
    Running,
    Stopped,
    Done(i32),
}

// Job impl
impl Job {
    pub fn new(id: usize, pgid: Pid, processes: Vec<Process>, command_line: String) -> Self { 
        Self { id, pgid, processes, status: JobStatus::Running, command_line }
    }

    /// Recompute aggregate status from the underlying processes.
    pub fn update_status(&mut self) {
        let all_done = self.processes.iter().all(|p| {
            matches!(p.status, ProcessStatus::Exited(_) | ProcessStatus::Signaled(_))
        });

        if all_done {
            let code = match self.processes.last().unwrap().status {
                ProcessStatus::Exited(c) => c,
                ProcessStatus::Signaled(sig) => 128 + sig as i32,
                _ => unreachable!(),
            };
            self.status = JobStatus::Done(code);
            return;
        }

        let any_stopped = self.processes.iter().any(|p| p.status == ProcessStatus::Stopped);
        let any_running = self.processes.iter().any(|p| p.status == ProcessStatus::Running);

        self.status = match (any_running, any_stopped) {
            (true, _) => JobStatus::Running,
            (false, true) => JobStatus::Stopped,
            _ => JobStatus::Running,
        };
    }

    /// Send a signal to the entire process group.
    pub fn send_signal(&self, sig: Signal) -> nix::Result<()> {
        // Negative pid targets the entire process group
        nix::sys::signal::kill(Pid::from_raw(-self.pgid.as_raw()), sig)
    }

    pub fn is_done(&self) -> bool {
        matches!(self.status, JobStatus::Done(_))
    }
}