use std::path::PathBuf;

use crate::{
    file_manager::canonicalize, mutant::MutationResult, runner::run_mutation_checks,
    test_runner::tests_successful,
};
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
    let path = check_path(&args.path)?;
    let file = check_file(&args.file, &path)?;
    run_mutation_checks(path, file)
}

fn check_path(source_folder_path: &String) -> Result<PathBuf, String> {
    let source_folder_path = canonicalize(&source_folder_path)?;

    // TODO Move all these to CLI
    if source_folder_path.is_file() {
        return Err("Path should be a folder file".to_string());
    }

    let scarb_toml = source_folder_path.join("Scarb.toml");
    if !scarb_toml.exists() {
        return Err("Scarb.toml file not found".to_string());
    }

    // Making sure all tests pass before starting
    if !tests_successful(&source_folder_path) {
        return Err("Tests aren't passing".to_string());
    }
    Ok(source_folder_path)
}

fn check_file(
    file: &Option<String>,
    source_folder_path: &PathBuf,
) -> Result<Option<PathBuf>, String> {
    if let Some(file) = file {
        let source_file_path = canonicalize(&file)?;
        if source_file_path.is_dir() {
            return Err(format!("{:?} should be a file", source_file_path));
        }

        // Assert file is within source_folder_path
        if !source_file_path.starts_with(source_folder_path) {
            return Err("File should be within the path folder".to_string());
        }

        // Assert extension is Cairo
        if source_file_path.extension().unwrap() != "cairo" {
            return Err("File extension should be .cairo".to_string());
        }
        Ok(Some(source_file_path))
    } else {
        Ok(None)
    }
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
