use std::process;

use colored::Colorize;

pub mod cli;
pub mod file_manager;
pub mod mutant;
pub mod runner;
pub mod test_runner;

fn main() {
    // TODO Ensure there is a Scarb cli?
    match cli::run() {
        Ok(end_msg) => {
            println!("{}", end_msg.green());
            process::exit(0);
        }
        Err(e) => {
            println!("{}", e.red());
            process::exit(1);
        }
    }
}
