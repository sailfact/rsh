use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction, signal};
use std::sync::atomic::{AtomicBool, Ordering};

static SIGCHLD_PENDING: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigchld(_: i32) {
    // Async-signal-safe: only set the flag. Reaping happens on the next
    // REPL tick via Shell::reap().
    SIGCHLD_PENDING.store(true, Ordering::Relaxed);
}

/// Return true if a SIGCHLD arrived since the last call, clearing the flag.
pub fn take_sigchld() -> bool {
    SIGCHLD_PENDING.swap(false, Ordering::Relaxed)
}

/// Install the shell's signal dispositions. Must be called once at startup,
/// before any child is forked.
pub fn install_shell_handlers(interactive: bool) {
    // SA_RESTART so a SIGCHLD arriving mid-readline doesn't surface as EINTR.
    let chld = SigAction::new(
        SigHandler::Handler(handle_sigchld),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    unsafe {
        let _ = sigaction(Signal::SIGCHLD, &chld);
    }

    if interactive {
        // The shell reclaims the terminal with tcsetpgrp while it is in a
        // background process group; without ignoring SIGTTOU/SIGTTIN the
        // kernel would stop the shell itself. SIGINT/SIGQUIT/SIGTSTP are
        // aimed at the foreground job, never the shell.
        unsafe {
            let _ = signal(Signal::SIGTTOU, SigHandler::SigIgn);
            let _ = signal(Signal::SIGTTIN, SigHandler::SigIgn);
            let _ = signal(Signal::SIGINT, SigHandler::SigIgn);
            let _ = signal(Signal::SIGQUIT, SigHandler::SigIgn);
            let _ = signal(Signal::SIGTSTP, SigHandler::SigIgn);
        }
    }
}

/// Restore default dispositions in a forked child before exec, so children
/// don't inherit the shell's ignored signals.
pub fn restore_default_handlers() {
    unsafe {
        for sig in [
            Signal::SIGCHLD,
            Signal::SIGTTOU,
            Signal::SIGTTIN,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTSTP,
        ] {
            let _ = signal(sig, SigHandler::SigDfl);
        }
    }
}
