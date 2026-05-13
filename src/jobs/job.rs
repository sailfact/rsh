use super::{Process, ProcessStatus};
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;

pub struct Job {
    pub id: usize,
    pub pgid: Pid, 
    pub processes: Vec<Process>,
    pub status: JobStatus,
}
pub enum JobStatus {
    Running,
    Stopped,
    Done(i32),
}

// Job impl
impl Job {
    pub fn wait(&mut self) {
        let mut exit_code = 0;
        for process in self.processes.iter_mut() {
            loop {
                match waitpid(process.pid, Some(WaitPidFlag::WUNTRACED)) {
                    Ok(WaitStatus::Exited(_, code)) => {
                        process.status = ProcessStatus::Exited(code);
                        exit_code = code;
                        break;
                    }
                    Ok(WaitStatus::Stopped(_sig, sig)) => {
                        process.status = ProcessStatus::Signaled(sig);
                        self.status = JobStatus::Stopped;
                        break;
                    }
                    Ok(WaitStatus::Signaled(_, sig, _)) => {
                        process.status = ProcessStatus::Signaled(sig);
                        exit_code = 128 + sig as i32;
                        break;
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("waitpid error: {}", e)
                    }
                }
            }
        }

        // if no processes stopped the job, it's done
        if !matches!(self.status, JobStatus::Stopped) {
            self.status = JobStatus::Done(exit_code)
        }
    }
    

    /// Send a signal to the entire process group.
    pub fn send_signal(&self, sig: Signal) {
        // negative pgid targets the entire process group
        let pgid = Pid::from_raw(-self.pgid.as_raw());
        if let Err(e) = kill(pgid, sig) {
            eprint!("failed to send signal to job {}: {}", self.id, e);
        }
    }

    
}