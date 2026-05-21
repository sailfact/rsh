#[allow(clippy::module_inception)]
pub mod parser;

pub use parser::Parser;

use crate::ast::{Command, Pipeline, Redirect};
