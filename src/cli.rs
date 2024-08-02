use crate::{mutant::MutationResult, runner::run_mutation_checks};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the folder containing the root Scarb.toml file
    #[arg(short, long)]
    path: String,
    /// Path to the Cairo file you want to mutate
    #[arg(short, long)]
    file: Option<String>,
}

// TODO Catch ctrl-c and clean
// TODO OPTION Add Clean command
// TODO OPTION Which mutation to apply
// TODO OPTION Limit threads to use?

// TODO later do an interactive CLI if missing args

pub fn run() -> Result<&'static str, String> {
    let args = Args::parse();
    run_mutation_checks(args.path, args.file)
}

// TODO Make this a map?
pub fn print_result(results: Vec<MutationResult>) -> Result<&'static str, String> {
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

    if failures.len() > 0 {
        println!("\nFailures:");

        for failure in &failures {
            println!("{}\n", failure);
        }
    }

    // for build_failure in &build_failures {
    //     println!("{}\n", build_failure);
    // }

    if failures.is_empty() {
        Ok("All mutation tests passed")
    } else {
        Err("Some mutation tests failed".to_string())
    }
}

fn s_or_nothing<T>(arr: &Vec<T>) -> &'static str {
    if arr.len() > 1 {
        "s"
    } else {
        ""
    }
}
