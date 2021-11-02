#![warn(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

pub use crate::cobalt::build;
pub use crate::cobalt::classify_path;
pub use crate::cobalt_model::Config;
pub use crate::error::Error;

pub mod cobalt_model;
pub mod error;

mod cobalt;
mod document;

mod pagination;
mod syntax_highlight;
mod custom_blocks;

pub use crate::syntax_highlight::{list_syntax_themes, list_syntaxes};
