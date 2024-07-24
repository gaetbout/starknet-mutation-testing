use crate::runner::run_mutation_checks;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the folder containing the root Scarb.toml file
    #[arg(short, long)]
    path: String,
}

// TODO Catch ctrl-c and clean
// TODO Add Clean command

// TODO later do an interactive CLI if missing args
// TODO Add a flag to limit threads to use?

pub fn run() -> Result<&'static str, &'static str> {
    let args = Args::parse();
    run_mutation_checks(args.path.to_owned())
}
