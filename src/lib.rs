#![feature(proc_macro_hygiene, decl_macro)]
#![forbid(unsafe_code)]

mod character;
mod config;
pub mod db;

pub mod bot;

pub use config::Config;
