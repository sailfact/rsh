use std::ffi::OsString;

pub const COMMANDS: &[&str] = &[
    "cat", "echo", "printf", "head", "tail", "wc", "sort",
    "uniq", "cut", "tr", "paste", "join", "fold", "fmt",
    "nl", "tac", "rev", "expand", "unexpand", "od", "xxd", "grep",
];

pub fn dispatch(name: &str, args: Vec<OsString>) -> i32 {
    match name {
        "cat"      => uu_cat::uumain(args.into_iter()),
        "echo"     => uu_echo::uumain(args.into_iter()),
        "printf"   => uu_printf::uumain(args.into_iter()),
        "head"     => uu_head::uumain(args.into_iter()),
        "tail"     => uu_tail::uumain(args.into_iter()),
        "wc"       => uu_wc::uumain(args.into_iter()),
        "sort"     => uu_sort::uumain(args.into_iter()),
        "uniq"     => uu_uniq::uumain(args.into_iter()),
        "cut"      => uu_cut::uumain(args.into_iter()),
        "tr"       => uu_tr::uumain(args.into_iter()),
        "paste"    => uu_paste::uumain(args.into_iter()),
        "join"     => uu_join::uumain(args.into_iter()),
        "fold"     => uu_fold::uumain(args.into_iter()),
        "fmt"      => uu_fmt::uumain(args.into_iter()),
        "tac"      => uu_tac::uumain(args.into_iter()),
        _          => unreachable!(),
    }
}