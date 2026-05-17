use std::ffi::OsString;

pub const COMMANDS: &[&str] = &[
    "seq", "factor", "basename", "dirname", "realpath",
    "pathchk", "link", "unlink", "sync", "truncate",
    "shuf", "comm", "csplit", "split", "tee",
    "test", "[", "expr", "xargs",
];

pub fn dispatch(name: &str, args: Vec<OsString>) -> i32 {
    match name {
        "seq"      => uu_seq::uumain(args.into_iter()),
        "factor"   => uu_factor::uumain(args.into_iter()),
        "basename" => uu_basename::uumain(args.into_iter()),
        "dirname"  => uu_dirname::uumain(args.into_iter()),
        "realpath" => uu_realpath::uumain(args.into_iter()),
        "pathchk"  => uu_pathchk::uumain(args.into_iter()),
        "link"     => uu_link::uumain(args.into_iter()),
        "unlink"   => uu_unlink::uumain(args.into_iter()),
        "sync"     => uu_sync::uumain(args.into_iter()),
        "truncate" => uu_truncate::uumain(args.into_iter()),
        "shuf"     => uu_shuf::uumain(args.into_iter()),
        "comm"     => uu_comm::uumain(args.into_iter()),
        "csplit"   => uu_csplit::uumain(args.into_iter()),
        "split"    => uu_split::uumain(args.into_iter()),
        "tee"      => uu_tee::uumain(args.into_iter()),
        "test"|"[" => uu_test::uumain(args.into_iter()),
        "expr"     => uu_expr::uumain(args.into_iter()),
        "xargs"    => uu_xargs::uumain(args.into_iter()),
        _          => unreachable!(),
    }
}