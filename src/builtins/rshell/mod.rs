// src/builtins/rshell/mod.rs
pub mod cd;
pub mod exit;
pub mod alias;
pub mod export;
pub mod source;
pub mod set;
pub mod unset;
pub mod fg;
pub mod bg;
pub mod ps;
pub mod jobs;
pub mod pwd;
pub mod kill;
pub mod wait;
pub mod exec;
pub mod trap;
pub mod umask;
pub mod read;
pub mod r#type;
pub mod hash;
pub mod history;



use std::sync::OnceLock;
use std::collections::HashMap;
use crate::ast::Command;
use crate::shell::Shell;

pub trait Builtin: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, args: &[String], shell: &mut Shell) -> i32;
}

pub struct Registry {
    builtins: HashMap<&'static str, Box<dyn Builtin>>,
}

impl Registry {
    pub fn new() -> Self {
        let mut r = Self { builtins: HashMap::new() };
        r.register(cd::Cd);
        r.register(exit::Exit);
        r.register(alias::Alias);
        r.register(alias::Unalias);
        r.register(source::Source);
        r.register(source::Dot);
        r.register(source::Return);
        r.register(export::Export);
        r.register(set::Set);
        r.register(set::Shift);
        r.register(unset::Unset);
        r.register(fg::Fg);
        r.register(bg::Bg);
        r.register(jobs::Jobs);
        r.register(pwd::Pwd);
        r.register(wait::Wait);
        r.register(kill::Kill);
        r.register(trap::Trap);
        r.register(umask::Umask);
        r.register(read::Read);
        r.register(r#type::Type);
        r.register(hash::Hash);
        r.register(history::History);
        r
    }

    fn register(&mut self, b: impl Builtin + 'static) {
        let name = b.name();
        self.builtins.insert(name, Box::new(b)); 
    }

    pub fn get(&self, name: &str) -> Option<&dyn Builtin> {
        self.builtins.get(name).map(|b| b.as_ref())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.builtins.contains_key(name)
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn global_registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::new)
}


pub fn dispatch(cmd: &Command, shell: &mut Shell) -> i32 {
    let name = cmd.argv[0].as_str();

    // Build the Registry and lookup the builtin
    match global_registry().get(name) {
        Some(builtin) => builtin.run(&cmd.argv, shell),
        None => { eprintln!("rsh: unknown builtin: {}", name); 1 }
    }
}