use crate::{
    file_manager::{change_line_content, collect_files_with_extension, copy_dir_all},
    test_runner::run_tests,
};
use rayon::prelude::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
enum Mutation {
    Equal,
    NotEqual,
    // TODO Some can lead to multiple modifications
    // GreaterThan, // e.g > => >= or <
    // GreaterThanOrEqual,
    // LessThan,
    // LessThanOrEqual,
}
struct Failure {
    error: &'static str,
    file: PathBuf,
    pos: usize,
}

pub fn run_mutation_checks(source_folder_path: String) -> Result<&'static str, &'static str> {
    let path_src = Path::new(source_folder_path.as_str());

    if !path_src.exists() {
        return Err("Invalid path doesn't exist");
    }

    if path_src.is_file() {
        return Err("Path should be a folder file");
    }

    let scarb_toml = path_src.join("Scarb.toml");
    if !scarb_toml.exists() {
        return Err("Scarb.toml file not found");
    }

    // Making sure all tests pass before starting
    if !run_tests(path_src) {
        return Err("Tests aren't passing");
    }

    let path_src = fs::canonicalize(&path_src).expect("Couldn't canonicalize path");
    find_and_test_mutations(&path_src.as_path(), "cli")
}

// TODO There must be a better way to return success or failure
fn find_and_test_mutations(path_src: &Path, subfolder: &str) -> Result<&'static str, &'static str> {
    // TODO This could be a map Mutation => data
    let mutations: Vec<(PathBuf, usize, String, Mutation)> = collect_mutations(path_src);
    if mutations.len() == 0 {
        return Ok("No mutations found");
    }

    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);

    let failures = mutations
        .into_par_iter()
        .enumerate()
        .filter_map(|(idx, (file, pos, original_line, mutation))| {
            let path_dst = &path_dst.join(idx.to_string());
            copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"])
                .expect("Couldn't copy test data");

            let (new_line, error) = match mutation {
                Mutation::Equal => (original_line.replace("==", "!="), "'==' updated to '!='"),
                Mutation::NotEqual => (original_line.replace("!=", "=="), "'!=' updated to '=='"),
            };

            let file_dst = path_dst.join(file.clone());
            change_line_content(&file_dst, pos + 1, &new_line).expect("Error applying mutation");
            if run_tests(path_dst) {
                Some(Failure {
                    error,
                    file: file.clone(),
                    pos,
                })
            } else {
                None
            }
        })
        .collect();
    fs::remove_dir_all(path_dst).expect("Error while removing tmp folder");
    print_result(failures)
}

fn print_result(failures: Vec<Failure>) -> Result<&'static str, &'static str> {
    if failures.len() > 0 {
        println!("Found {} failing mutation(s):", failures.len());
        for failure in failures {
            println!("\tMutation applied {}", failure.error);
            println!("\tFile {:?} line {:?}\n", failure.file, failure.pos + 1);
        }
        Err("Some mutation tests failed")
    } else {
        Ok("All mutation tests passed")
    }
}

fn collect_mutations(path_src: &Path) -> Vec<(PathBuf, usize, String, Mutation)> {
    let files = collect_files_with_extension(path_src, "cairo").expect("Couldn't collect files");

    let mut mutations = Vec::new();

    for file in &files {
        // Read the content of the file into a string
        let content = fs::read_to_string(&file).expect("Error while reading the file");
        // Look for mutation
        for (pos, line) in content.lines().into_iter().enumerate() {
            let line = line.to_string();
            if line.contains("==") {
                mutations.push((
                    file.strip_prefix(path_src).expect("msg").to_path_buf(),
                    pos,
                    line.clone(),
                    Mutation::Equal,
                ));
            }

            if line.contains("!=") {
                mutations.push((
                    file.strip_prefix(path_src).expect("msg").to_path_buf(),
                    pos,
                    line.clone(),
                    Mutation::NotEqual,
                ));
            }
        }
    }
    mutations
}

#[cfg(test)]
mod tests {
    use super::find_and_test_mutations;
    use std::path::Path;
    #[test]
    fn test_equal() {
        assert!(find_and_test_mutations(Path::new("test_data/equal"), "equal").is_ok());
    }

    #[test]
    fn test_equal_fail() {
        assert!(find_and_test_mutations(Path::new("test_data/equalFail"), "equalFail").is_err());
    }

    #[test]
    fn test_not_equal() {
        assert!(find_and_test_mutations(Path::new("test_data/notEqual"), "notEqual").is_ok());
    }

    #[test]
    fn test_not_equal_fail() {
        assert!(
            find_and_test_mutations(Path::new("test_data/notEqualFail"), "notEqualFail").is_err()
        );
    }
}
