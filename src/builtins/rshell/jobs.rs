use super::Builtin;
use crate::jobs::job::JobStatus;
use crate::shell::Shell;

pub struct Jobs;

impl Builtin for Jobs {
    fn name(&self) -> &'static str {
        "jobs"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let long = args.iter().any(|a| a == "-l");

        for job in &shell.jobs {
            let status = match &job.status {
                JobStatus::Running => "Running",
                JobStatus::Stopped => "Stopped",
                JobStatus::Done(_) => "Done",
            };
            if long {
                println!(
                    "[{}] {:7}  {:6}  {}",
                    job.id,
                    status,
                    job.pgid,
                    job.argv_string()
                );
            } else {
                println!("[{}] {:7}  {}", job.id, status, job.argv_string());
            }
        }
        0
    }
}
