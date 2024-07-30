use crate::{mutant::MutationResult, runner::run_mutation_checks};
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

// TODO Make this a map?
pub fn print_result(results: Vec<MutationResult>) -> Result<&'static str, &'static str> {
    // TODO Add some color in this result?
    println!(
        "Found {} mutation{}:",
        results.len(),
        s_or_nothing(&results)
    );
    println!(
        "\t{} successful",
        results
            .iter()
            .filter(|r| matches!(r, MutationResult::Success(_)))
            .count()
    );
    let build_failures = results
        .iter()
        .filter(|r| matches!(r, MutationResult::BuildFailure(_)))
        .collect::<Vec<_>>();
    println!("\t{} build failures", build_failures.len());

    let failures = results
        .iter()
        .filter(|r| matches!(r, MutationResult::Failure(_)))
        .collect::<Vec<_>>();
    println!("\t{} failures", failures.len());

    println!("\nFailures:");

    for failure in &failures {
        println!("{}\n", failure);
    }

    // for build_failure in &build_failures {
    //     println!("{}\n", build_failure);
    // }

    if failures.is_empty() {
        Ok("All mutation tests passed")
    } else {
        Err("Some mutation tests failed")
    }
}

fn s_or_nothing<T>(arr: &Vec<T>) -> &'static str {
    if arr.len() > 1 {
        "s"
    } else {
        ""
    }
}
