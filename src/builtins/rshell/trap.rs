use super::Builtin;
use crate::shell::Shell;

pub struct Trap;

impl Builtin for Trap {
    fn name(&self) -> &'static str {
        "trap"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        if args.len() < 2 {
            // print all set traps
            let mut entries: Vec<_> = shell.traps.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            for (sig, cmd) in entries {
                println!("trap -- '{}' {}", cmd, sig_name(*sig));
            }
            return 0;
        }

        // trap - SIGNAL [...] → reset to default
        if args[1] == "-" {
            for sig_name_str in &args[2..] {
                if let Some(n) = sig_number(sig_name_str) {
                    shell.traps.remove(&n);
                } else {
                    eprintln!("trap: {}: invalid signal", sig_name_str);
                    return 1;
                }
            }
            return 0;
        }

        // trap ACTION SIGNAL [SIGNAL...]
        if args.len() < 3 {
            eprintln!("trap: usage: trap action signal [signal ...]");
            return 1;
        }

        let action = &args[1];
        let mut status = 0;
        for sig_name_str in &args[2..] {
            match sig_number(sig_name_str) {
                Some(n) => {
                    shell.traps.insert(n, action.clone());
                }
                None => {
                    eprintln!("trap: {}: invalid signal", sig_name_str);
                    status = 1;
                }
            }
        }
        status
    }
}

fn sig_number(name: &str) -> Option<i32> {
    // try plain number
    if let Ok(n) = name.parse::<i32>() {
        return Some(n);
    }
    let upper = name.to_uppercase();
    let canon = if upper.starts_with("SIG") {
        upper.clone()
    } else {
        format!("SIG{}", upper)
    };
    match canon.as_str() {
        "SIGHUP" => Some(1),
        "SIGINT" => Some(2),
        "SIGQUIT" => Some(3),
        "SIGKILL" => Some(9),
        "SIGTERM" => Some(15),
        "SIGSTOP" => Some(19),
        "SIGCONT" => Some(18),
        "SIGTSTP" => Some(20),
        "SIGPIPE" => Some(13),
        "SIGALRM" => Some(14),
        "SIGUSR1" => Some(10),
        "SIGUSR2" => Some(12),
        "SIGCHLD" => Some(17),
        "EXIT" => Some(0),
        _ => None,
    }
}

fn sig_name(n: i32) -> String {
    match n {
        0 => "EXIT".to_string(),
        1 => "HUP".to_string(),
        2 => "INT".to_string(),
        3 => "QUIT".to_string(),
        9 => "KILL".to_string(),
        13 => "PIPE".to_string(),
        14 => "ALRM".to_string(),
        15 => "TERM".to_string(),
        17 => "CHLD".to_string(),
        18 => "CONT".to_string(),
        19 => "STOP".to_string(),
        20 => "TSTP".to_string(),
        10 => "USR1".to_string(),
        12 => "USR2".to_string(),
        _ => n.to_string(),
    }
}
