# Commands
## Shell Builtins
| Command                   | Description                                               |          
|---------------------------|-----------------------------------------------------------|
|```cd```                   | Changes the shell's own working directory                 |    
|```exit```                 | Terminates the shell process                              |
| ```alias```               | Mutates shell.aliases                                     |
|```export```               | Mutates shell.env, inherited by children                  |
|```unset```                | Removes vars from shell.env                               |
|```set```                  | Sets shell options / positional params                    |
|```source``` / ```.```     | Runs a script in the current shell context                |
|```exec```                 | Replaces the shell process image                          |
|```jobs```                 | Reads shell.jobs                                          |
|```fg```                   | Sends SIGCONT, calls tcsetpgrp, waits                     |
| ```bg```                  | Sends SIGCONT, leaves in background                       |
|```wait```                 | Waits on a specific job/pid                               |
|```trap```                 | Registers signal handlers in the shell                    |
|```umask```                | Gets/sets the file creation mask                          |
|```read```                 | Reads a line from stdin into a shell variable             |
|```echo```                 | Can be builtin for speed (but uutils is fine too)         |
|```true```/```false```     | Return  0 / 1, trivial to inline                          |
|```pwd```                  | Can read env::current_dir() in-process                    |
|```shift```                | Shifts positional parameters                              |
|```break```/```continue``` | Loop control (if you add scripting)                       |
|```return```               | Returns from a function/source                            |
|```type```                 | Resolves whether a name is builtin, alias, or external    |
|```hash```                 | Caches PATH lookups                                       |
|```history```              | Reads/writes readline history                             |

## uutils Coreutils (via uu_* crates or PATH)
These are safe to fork() + run in-process as library calls. Grouped by what you'll actually reach for:
### File operations 
|Crate            | Commands        |
|-----------------|-----------------|
|```uu_ls```      |```ls```         |
|```uu_cp```      |```cp```         |
|```uu_mv```      |```mv```         |
|```uu_rm```      |```rm```         |
|```uu_mkdir```   |```mkdir```      |
|```uu_rmdir```   |```rmdir```      |
|```uu_touch```   |```touch```      |
|```uu_ln```      |```ln```         |
|```uu_chmod```   |```chmod```      |
|```uu_chown```   |```chown```      |
|```uu_stat```    |```stat```       |
|```uu_du```      |```du```         |
|```uu_df```      |```df```         |
|```uu_install``` |```install```    |
|```uu_mktemp```  |```mktemp```     |
#### Cargo.toml
```toml
[dependencies]
uu_ls    = "0.0.28"
uu_cat   = "0.0.28"
uu_cp    = "0.0.28"
uu_rm    = "0.0.28"
uu_mkdir = "0.0.28"
uu_echo  = "0.0.28"
# etc. — pick only what you need
uucore   = "0.0.28"
```
#### command.rs
```rust
const UUTILS_COREUTILS_COMMANDS: &[&str] = &[
    "cp", 
    "mv",
    "rm",
    "mkdir",
    "rmdir",
    "touch",
    "ln",
    "chmod",
    "chown",
    "stat",
    "du",
    "df",
    "install",
    "mktemp",    
];
```

### Text processing
|Crate  |Commands|
|---|---|
|```uu_cat```|```cat```|
|```uu_echo```|```echo```|
|```uu_printf```|```printf```|
|```uu_head```|```head```|
|```uu_tail```|```tail```|
|```uu_wc```|```wc```|
|```uu_sort```|```sort```|
|```uu_uniq```|```uniq```|
|```uu_cut```|```cut```|
|```uu_tr```|```tr```|
|```uu_paste```|```paste```|
|```uu_join```|```join```|
|```uu_fold```|```fold```|
|```uu_fmt```|```fmt```|
|```uu_n```|```nl```|
|```uu_tac```|```tac```|
|```uu_rev```|```rev```|
|```uu_expand```|```expand``` (tabs→spaces)|
|```uu_unexpand```|```unexpand```|
|```uu_od```|```od```|
|```uu_xxd```|```xxd```|
|```uu_grep```|```grep``` (uutils has this but it's separate — uutils/grep)|

### System / process info
|Crate|Commands|
|-----|--------|
|`uu_whoami`|`whoami`|
|`uu_id`|`id`|
|`uu_hostname`|`hostname`|
|`uu_uname`|`uname`|
|`uu_uptime`|`uptime`|
|`uu_date`|`date`|
|`uu_sleep`|`sleep`|
|`uu_yes`|`yes`|
|`uu_env`|`env` (run command with modified environment)|
|`uu_nohup`|`nohup`|
|`uu_timeout`|`timeout`|
|`uu_nice`|`nice`|
|`uu_kill`|`kill` (note: conflicts with shell builtin kill — you may want to keep your own)|
|`uu_printenv`|`printenv`|
|`uu_tty`|`tty`|
|`uu_stty`|`stty`|
### Hashing / encoding
|Crate|Commands|
|---|---|
|`uu_base32`|`base32`|
|`uu_base64`|`base64`|
|`uu_md5sum`|`md5sum`|
|`uu_sha256sum`|`sha256sum`|
|`uu_sha512sum`|`sha512sum`|
|`uu_cksum`|`cksum`|
|`uu_sum`|`sum`|
|`uu_b2sum`|`b2sum`|
### Miscellaneous
|Crate|Commands|
|---|---|
|`uu_seq`|`seq`|
|`uu_factor`|`factor`|
|`uu_basename`|`basename`|
|`uu_dirname`|`dirname`|
|`uu_realpath`|`realpath`|
|`uu_pathchk`|`pathchk`|
|`uu_link` / `uu_unlink`|`link` / `unlink`|
|`uu_sync`|`sync`|
|`uu_truncate`|`truncate`|
|`uu_shuf`|`shuf`|
|`uu_comm`|`comm`|
|`uu_csplit`|`csplit`|
|`uu_split`|`split`|
|`uu_tee`|`tee` (great one to embed — interacts with your pipe setup)|
|`uu_xargs`|xargs (in uutils/findutils)|
|`uu_test` / `uu_[`|`test` / `[` (essential for scripting support)|
|`uu_expr`|expr|

## Priority Order
| # | Priority | Commands |
|---|----------|----------|
|**1**|Must-have shell builtins|cd, exit, alias, export, unset, source, jobs, fg, bg, wait, trap, read, type, pwd|
|**2**| High-value uutils| ls, cat, echo, cp, mv, rm, mkdir, grep, head, tail, wc, sort|
|**3**| Scripting support| test, [, expr, true, false, sleep, printf|
|**4**| Nice to have|tee, tac, cut, tr, env, timeout, xargs