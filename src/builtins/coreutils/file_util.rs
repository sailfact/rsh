use std::ffi::OsString;

pub const COMMANDS: &[&str] = &[
    "cp", "mv", "rm", "mkdir", "rmdir", "touch", "ln", "chmod", "chown", "stat", "du", "df",
    "install", "mktemp",
];

pub fn dispatch(name: &str, args: Vec<OsString>) -> i32 {
    match name {
        "cp" => uu_cp::uumain(args.into_iter()),
        "mv" => uu_mv::uumain(args.into_iter()),
        "rm" => uu_rm::uumain(args.into_iter()),
        "mkdir" => uu_mkdir::uumain(args.into_iter()),
        "rmdir" => uu_rmdir::uumain(args.into_iter()),
        "touch" => uu_touch::uumain(args.into_iter()),
        "ln" => uu_ln::uumain(args.into_iter()),
        "chmod" => uu_chmod::uumain(args.into_iter()),
        "chown" => uu_chown::uumain(args.into_iter()),
        "stat" => uu_stat::uumain(args.into_iter()),
        "du" => uu_du::uumain(args.into_iter()),
        "df" => uu_df::uumain(args.into_iter()),
        "install" => uu_install::uumain(args.into_iter()),
        "mktemp" => uu_mktemp::uumain(args.into_iter()),
        _ => unreachable!(),
    }
}
