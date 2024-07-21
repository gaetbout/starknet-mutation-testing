use std::process;

pub mod cli;
pub mod file_manager;

fn main() {
    // TODO Ensure there is a Scarb cli?
    match cli::run() {
        Ok(end_msg) => {
            println!("{}", end_msg);
            process::exit(0);
        }
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    }
}
