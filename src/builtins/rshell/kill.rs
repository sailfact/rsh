use super::Builtin;
use crate::shell::Shell;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

pub struct Kill;

impl Builtin for Kill {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            eprintln!("kill: usage: kill [-SIGNAL] pid/%job [...]");
            return 1;
        }

        // kill -l → list signal names
        if args.get(1).map(String::as_str) == Some("-l") {
            print_signal_list();
            return 0;
        }

        // Parse optional signal, then targets
        let mut iter = args[1..].iter().peekable();
        let mut sig = Signal::SIGTERM;

        if let Some(first) = iter.peek().filter(|f| f.starts_with('-')) {
            let s = first.strip_prefix('-').unwrap_or(first);
            match parse_signal(s) {
                Some(parsed) => {
                    sig = parsed;
                    iter.next();
                }
                None => {
                    eprintln!("kill: {}: invalid signal specification", s);
                    return 1;
                }
            }
        }

        let mut status = 0;
        for target in iter {
            if target.starts_with('%') {
                // job spec
                match shell.find_job_mut(Some(target.as_str())) {
                    None => {
                        eprintln!("kill: {}: no such job", target);
                        status = 1;
                    }
                    Some(job) => {
                        job.send_signal(sig);
                    }
                }
            } else {
                // PID
                match target.parse::<i32>() {
                    Ok(pid) => {
                        if let Err(e) = kill(Pid::from_raw(pid), sig) {
                            eprintln!("kill: ({}): {}", pid, e);
                            status = 1;
                        }
                    }
                    Err(_) => {
                        eprintln!("kill: {}: invalid target", target);
                        status = 1;
                    }
                }
            }
        }
        status
    }
}

fn parse_signal(s: &str) -> Option<Signal> {
    // try numeric first
    if let Ok(n) = s.parse::<i32>() {
        return Signal::try_from(n).ok();
    }
    // try name (uppercase)
    let upper = s.to_uppercase();
    let name = if upper.starts_with("SIG") {
        upper.clone()
    } else {
        format!("SIG{}", upper)
    };
    match name.as_str() {
        "SIGHUP" => Some(Signal::SIGHUP),
        "SIGINT" => Some(Signal::SIGINT),
        "SIGQUIT" => Some(Signal::SIGQUIT),
        "SIGKILL" => Some(Signal::SIGKILL),
        "SIGTERM" => Some(Signal::SIGTERM),
        "SIGSTOP" => Some(Signal::SIGSTOP),
        "SIGCONT" => Some(Signal::SIGCONT),
        "SIGTSTP" => Some(Signal::SIGTSTP),
        "SIGPIPE" => Some(Signal::SIGPIPE),
        "SIGALRM" => Some(Signal::SIGALRM),
        "SIGUSR1" => Some(Signal::SIGUSR1),
        "SIGUSR2" => Some(Signal::SIGUSR2),
        "SIGCHLD" => Some(Signal::SIGCHLD),
        _ => None,
    }
}

fn print_signal_list() {
    let sigs = [
        (1, "HUP"),
        (2, "INT"),
        (3, "QUIT"),
        (4, "ILL"),
        (6, "ABRT"),
        (8, "FPE"),
        (9, "KILL"),
        (11, "SEGV"),
        (13, "PIPE"),
        (14, "ALRM"),
        (15, "TERM"),
        (17, "CHLD"),
        (18, "CONT"),
        (19, "STOP"),
        (20, "TSTP"),
        (10, "USR1"),
        (12, "USR2"),
    ];
    for (n, name) in &sigs {
        print!("{}) SIG{}\t", n, name);
    }
    println!();
}
