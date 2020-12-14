#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod character;
mod config;
mod dice;

pub mod bot;
pub mod db;

pub use config::Config;
pub use dice::{Dice, D10, D20, D6};
