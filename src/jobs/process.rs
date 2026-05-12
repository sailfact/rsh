use super::{Pid, Signal};
pub struct Process {
    pub pid: Pid,
    pub argv: Vec<String>,
    pub status: ProcessStatus,
}

#[derive(PartialEq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Signaled(Signal),
    Stopped,
}