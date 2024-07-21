use clap::Parser;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::file_manager::{change_line_content, collect_files_with_extension, copy_dir_all};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the folder containing the root Scarb.toml file
    #[arg(short, long)]
    path: String,
}

// TODO later do a interactive CLI if missing args

pub fn run() -> Result<&'static str, &'static str> {
    let args = Args::parse();
    do_stuff(&args.path.to_owned(), "cli")
}

enum Mutation {
    Equal,
    NotEqual,
    // TODO Some can lead to multiple modifications
    // GreaterThan, // e.g > => >= or <
    // GreaterThanOrEqual,
    // LessThan,
    // LessThanOrEqual,
}

// This should return an error
fn do_stuff(folder_path: &str, subfolder: &str) -> Result<&'static str, &'static str> {
    let path_src = Path::new(folder_path);

    // Check there is a Scarb.toml file
    let scarb_toml = path_src.join("Scarb.toml");
    assert!(scarb_toml.exists(), "Scarb.toml file not found");

    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);
    copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"]).expect("Couldn't copy test data");

    // First ensure all tests pass
    // Run the scarb test command
    assert!(run_tests(path_dst), "Pre state tests failed");

    let files = collect_files_with_extension(path_dst, "cairo").expect("Couldn't collect files");

    // TODO This could be a map Mutation => data
    // TODO And could start a thread for each mutation type
    // Test each mutant in parallel.
    let mutations = collect_mutations(files);
    println!("Mutations found: {}", mutations.len());

    let mut failures = Vec::new();
    // Mutate the file
    for (file, pos, original_line, mutation) in mutations {
        let (new_line, error) = match mutation {
            Mutation::Equal => (original_line.replace("==", "!="), "'==' updated to '!='"),
            Mutation::NotEqual => (original_line.replace("!=", "=="), "'!=' updated to '=='"),
        };

        change_line_content(&file, pos + 1, &new_line).expect("Error applying mutation");
        if run_tests(path_dst) {
            failures.push((error, file.clone(), pos));
        }
        change_line_content(&file, pos + 1, &original_line).expect("Error reverting content");
    }

    fs::remove_dir_all(path_dst).expect("Error while removing tmp folder");

    if failures.len() > 0 {
        println!("Found {} failing mutation(s):", failures.len());
        for (error, file, pos) in failures {
            println!("\tMutation applied {}", error);
            println!("\tFile {:?} line {:?}\n", file, pos + 1);
        }
        Err("Some mutation tests failed")
    } else {
        Ok("All mutation tests passed")
    }
}

fn collect_mutations(files: Vec<PathBuf>) -> Vec<(PathBuf, usize, String, Mutation)> {
    let mut mutations = Vec::new();

    // Print the collected files
    for file in &files {
        // Read the content of the file into a string
        let content = fs::read_to_string(&file).expect("Error while reading the file");
        // Look for mutation
        for (pos, line) in content.lines().into_iter().enumerate() {
            let line = line.to_string();
            if line.contains("==") {
                mutations.push((file.clone(), pos, line.clone(), Mutation::Equal));
            }

            if line.contains("!=") {
                mutations.push((file.clone(), pos, line.clone(), Mutation::NotEqual));
            }
        }
    }
    mutations
}

// TODO Do a TestRunner to support other tests frameworks
fn run_tests(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("test")
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        // let stdout = String::from_utf8_lossy(&output.stdout);
        // println!("Command output: {}", stdout);
    } else {
        // let stderr = String::from_utf8_lossy(&output.stderr);
        // let stdout = String::from_utf8_lossy(&output.stdout);
        // // println!("{}", stdout); // TODO Parse the failing tests
        // println!("Command failed with error: {}", stderr);
    }
    output.status.success()
}

#[cfg(test)]
mod tests {
    use super::do_stuff;
    // TODO Should add test that ensure the mutation is detected
    #[test]
    fn test_equal() {
        do_stuff("test_data/equal", "equal");
    }

    #[test]
    fn test_equal_fail() {
        do_stuff("test_data/equalFail", "equalFail");
    }

    #[test]
    fn test_not_equal() {
        do_stuff("test_data/notEqual", "notEqual");
    }

    #[test]
    fn test_not_equal_fail() {
        do_stuff("test_data/notEqualFail", "notEqualFail");
    }
}
