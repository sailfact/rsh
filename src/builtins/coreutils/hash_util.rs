use std::ffi::OsString;

pub const COMMANDS: &[&str] = &[
    "base32",
    "base64",
    "md5sum",
    "sha256sum",
    "sha512sum",
    "cksum",
    "sum",
    "b2sum",
];

pub fn dispatch(name: &str, args: Vec<OsString>) -> i32 {
    match name {
        "base32" => uu_base32::uumain(args.into_iter()),
        "base64" => uu_base64::uumain(args.into_iter()),
        "md5sum" => uu_md5sum::uumain(args.into_iter()),
        "sha256sum" => uu_sha256sum::uumain(args.into_iter()),
        "sha512sum" => uu_sha512sum::uumain(args.into_iter()),
        "cksum" => uu_cksum::uumain(args.into_iter()),
        "sum" => uu_sum::uumain(args.into_iter()),
        "b2sum" => uu_b2sum::uumain(args.into_iter()),
        _ => unreachable!(),
    }
}
