use super::Builtin;
use crate::shell::Shell;

pub struct Umask;

impl Builtin for Umask {
    fn name(&self) -> &'static str {
        "umask"
    }

    fn run(&self, args: &[String], shell: &mut Shell) -> i32 {
        let symbolic = args.iter().any(|a| a == "-S");

        if args.len() == 1 || (args.len() == 2 && symbolic) {
            if symbolic {
                println!("{}", symbolic_mask(shell.umask));
            } else {
                println!("{:04o}", shell.umask);
            }
            return 0;
        }

        // find the mask argument (skip flags)
        let mask_arg = args[1..].iter().find(|a| *a != "-S");
        match mask_arg {
            None => {
                println!("{:04o}", shell.umask);
                0
            }
            Some(s) => match u32::from_str_radix(s, 8) {
                Ok(mask) => {
                    shell.umask = mask & 0o777;
                    unsafe {
                        libc::umask(shell.umask as libc::mode_t);
                    }
                    0
                }
                Err(_) => {
                    eprintln!("umask: {}: invalid octal number", s);
                    1
                }
            },
        }
    }
}

fn symbolic_mask(mask: u32) -> String {
    let perm = !mask & 0o777;
    let bits = |shift: u32| -> String {
        let r = if (perm >> (shift + 2)) & 1 == 1 {
            'r'
        } else {
            '-'
        };
        let w = if (perm >> (shift + 1)) & 1 == 1 {
            'w'
        } else {
            '-'
        };
        let x = if (perm >> shift) & 1 == 1 { 'x' } else { '-' };
        format!("{}{}{}", r, w, x)
    };
    format!("u={},g={},o={}", bits(6), bits(3), bits(0))
}
