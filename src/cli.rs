use std::{fs, path::PathBuf};

use crate::{
    file_manager::{canonicalize, get_tmp_dir},
    mutant::MutationResult,
    runner::run_mutation_checks,
    test_runner::tests_successful,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Mandatory group of any arguments
    #[clap(flatten)]
    group: Group,
    /// Path to the Cairo file you want to mutate
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Group {
    /// Path to the folder containing the root Scarb.toml file
    #[clap(short, long)]
    path: Option<String>,
    /// Used to clean the generated files after a crash
    #[clap(short, long)]
    clean: bool,
}
use crate::Result;

// TODO Catch ctrl-c and clean
// TODO OPTION Which mutation to apply
// TODO OPTION Limit threads to use?

// TODO later do an interactive CLI if missing args

pub fn run() -> Result<()> {
    let args = Args::parse();
    if args.group.clean {
        let tmp_dir = get_tmp_dir();
        if !tmp_dir.exists() {
            // TODO Turn all these println into a logger
            println!("Nothing to clean");
            return Ok(());
        }
        fs::remove_dir_all(tmp_dir).expect("Error while removing tmp folder");
        println!("Cleaned");
        return Ok(());
    }
    let path = check_path(&args.group.path.unwrap())?;
    let file = check_file(args.file, &path)?;
    run_mutation_checks(path, file)
}

fn check_path(source_folder_path: &String) -> Result<PathBuf> {
    let source_folder_path = canonicalize(&source_folder_path)?;

    if source_folder_path.is_file() {
        return Err("Path should be a folder file".into());
    }

    let scarb_toml = source_folder_path.join("Scarb.toml");
    if !scarb_toml.exists() {
        return Err("Scarb.toml file not found".into());
    }

    // Making sure all tests pass before starting
    if !tests_successful(&source_folder_path) {
        return Err("Tests aren't passing".into());
    }
    Ok(source_folder_path)
}

fn check_file(file: Option<String>, source_folder_path: &PathBuf) -> Result<Option<PathBuf>> {
    if let Some(file) = file {
        let source_file_path = canonicalize(&file)?;
        if source_file_path.is_dir() {
            return Err(format!("{:?} should be a file", source_file_path).into());
        }

        // Assert file is within source_folder_path
        if !source_file_path.starts_with(source_folder_path) {
            return Err("File should be within the path folder".into());
        }

        // Assert extension is Cairo
        if source_file_path.extension().unwrap() != "cairo" {
            return Err("File extension should be .cairo".into());
        }
        Ok(Some(source_file_path))
    } else {
        Ok(None)
    }
}
// TODO Make this a map?
pub fn print_result(results: Vec<MutationResult>) -> Result<()> {
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
        println!("All mutation tests passed");
        Ok(())
    } else {
        Err("Some mutation tests failed".into())
    }
}

fn s_or_nothing<T>(arr: &Vec<T>) -> &'static str {
    if arr.len() > 1 {
        "s"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;

    #[test]
    fn test_s_or_nothing() {
        let arr = vec![1, 2, 3];
        assert_eq!(s_or_nothing(&arr), "s");

        let arr = vec![1];
        assert_eq!(s_or_nothing(&arr), "");
    }

    #[test]
    fn test_check_path() {
        let path = "./test_data/assert/".to_string();
        assert!(check_path(&path).is_ok());

        let path = "./doesnotexist".to_string();
        assert_eq!(
            check_path(&path).unwrap_err(),
            Error::FsInvalidPath { path }
        );

        let path = "./test_data/assert/Scarb.toml".to_string();
        assert!(check_path(&path)
            .unwrap_err()
            .to_string()
            .contains("Path should be a folder file"));
    }

    #[test]
    fn test_check_file() {
        let dst = canonicalize(&"./test_data/assert".to_string()).unwrap();
        let path = "./test_data/assert/src/lib.cairo".to_string();
        assert!(check_file(None, &dst).is_ok());
        assert!(check_file(Some(path), &dst).is_ok());

        let path = "./test_data".to_string();
        assert!(check_file(Some(path), &dst)
            .unwrap_err()
            .to_string()
            .contains("should be a file"));

        let path = "./test_data/equal/Scarb.toml".to_string();
        assert!(check_file(Some(path), &dst)
            .unwrap_err()
            .to_string()
            .contains("File should be within the path folder"));

        let path = "./test_data/assert/Scarb.toml".to_string();
        assert!(check_file(Some(path), &dst)
            .unwrap_err()
            .to_string()
            .contains("File extension should be .cairo"));
    }
}
