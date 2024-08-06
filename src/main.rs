use std::process;

mod error;
pub use error::{Error, Result};

pub mod cli;
pub mod file_manager;
pub mod mutant;
pub mod runner;
pub mod test_runner;

fn main() {
    // TODO Ensure there is a Scarb cli?
    match cli::run() {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(1);
        }
    }
}
