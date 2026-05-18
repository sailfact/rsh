use std::ffi::OsString;

// Note: "kill" omitted — keep as a shell builtin using nix signals directly
pub const COMMANDS: &[&str] = &[
    "whoami", "id", "hostname", "uname", "uptime", "date",
    "sleep", "yes", "env", "nohup", "timeout", "nice",
    "printenv", "tty", "stty",
];

pub fn dispatch(name: &str, args: Vec<OsString>) -> i32 {
    match name {
        "whoami"   => uu_whoami::uumain(args.into_iter()),
        "id"       => uu_id::uumain(args.into_iter()),
        "hostname" => uu_hostname::uumain(args.into_iter()),
        "uname"    => uu_uname::uumain(args.into_iter()),
        "uptime"   => uu_uptime::uumain(args.into_iter()),
        "date"     => uu_date::uumain(args.into_iter()),
        "sleep"    => uu_sleep::uumain(args.into_iter()),
        "yes"      => uu_yes::uumain(args.into_iter()),
        "env"      => uu_env::uumain(args.into_iter()),
        "nohup"    => uu_nohup::uumain(args.into_iter()),
        "timeout"  => uu_timeout::uumain(args.into_iter()),
        "nice"     => uu_nice::uumain(args.into_iter()),
        "printenv" => uu_printenv::uumain(args.into_iter()),
        "tty"      => uu_tty::uumain(args.into_iter()),
        "stty"     => uu_stty::uumain(args.into_iter()),
        _          => unreachable!(),
    }
}