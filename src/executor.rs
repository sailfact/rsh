pub mod executor {
    use crate::Shell;
    use crate::jobs::{Job, JobStatus, Process, ProcessStatus};
    use crate::parser::{Pipeline, Command};
    use nix::unistd::Pid;

    pub fn execute(shell: &mut Shell, pipeline: Pipeline) -> i32{
        
    }   

    fn spawn_pipeline(cmds: &[Command]) -> i32 {
        // Todo
        0
    }

    fn wait_foreground(pgid: Pid) -> i32 {
        // Todo
        0
    }
}