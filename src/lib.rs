#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(fs_try_exists)]
#![feature(absolute_path)]
#![feature(path_file_prefix)]
#![allow(dead_code)]

#[macro_use]
pub mod macros;

pub mod util;
pub mod cache;
pub mod options;
pub mod repository;
// pub mod degit;
pub mod traits;
pub mod prelude;
pub mod app;

// pub use degit::*;
pub use prelude::*;

pub use colored::Colorize;
