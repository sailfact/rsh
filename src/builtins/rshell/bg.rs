use crate::shell::Shell;
use crate::jobs::job::JobStatus;
use nix::sys::signal::Signal;
use super::Builtin;

pub struct Bg;

impl Builtin for Bg {
    fn name(&self) -> &'static str { "bg" }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let spec = args.get(1).map(String::as_str);

        match shell.find_job_mut(spec) {
            None => {
                eprintln!("bg: no such job");
                1
            }
            Some(job) => {
                let pgid = job.pgid;
                let argv = job.argv_string();
                job.status = JobStatus::Running;

                if let Err(e) = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(-pgid.as_raw()),
                    Signal::SIGCONT,
                ) {
                    eprintln!("bg: SIGCONT failed: {}", e);
                    return 1;
                }

                eprintln!("[?] {}", argv);
                0
            }
        }
    }
}
