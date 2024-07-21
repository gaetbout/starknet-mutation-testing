use crate::{
    file_manager::{change_line_content, collect_files_with_extension, copy_dir_all},
    test_runner::run_tests,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

enum Mutation {
    Equal,
    NotEqual,
    // TODO Some can lead to multiple modifications
    // GreaterThan, // e.g > => >= or <
    // GreaterThanOrEqual,
    // LessThan,
    // LessThanOrEqual,
}

pub fn run_mutation_checks(source_folder_path: String) -> Result<&'static str, &'static str> {
    let path_src = Path::new(source_folder_path.as_str());

    if !path_src.exists() {
        return Err("Incorrect pre state: Path doesn't exist");
    }

    if path_src.is_file() {
        return Err("Incorrect pre state: Path should be a folder file");
    }

    let scarb_toml = path_src.join("Scarb.toml");
    if !scarb_toml.exists() {
        return Err("Incorrect pre state: Scarb.toml file not found");
    }

    // Making sure all tests pass before starting
    if !run_tests(path_src) {
        return Err("Incorrect pre state: tests aren't passing");
    }

    test_mutations(&path_src, "cli")
}

fn test_mutations(path_src: &Path, subfolder: &str) -> Result<&'static str, &'static str> {
    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);

    copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"]).expect("Couldn't copy test data");

    // First ensure all tests pass
    // Run the scarb test command

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

#[cfg(test)]
mod tests {
    use super::do_stuff;
    // TODO Should add test that ensure the mutation is detected
    use std::path::Path;
    #[test]
    fn test_equal() {
        do_stuff(Path::new("test_data/equal"), "equal").unwrap();
    }

    #[test]
    fn test_equal_fail() {
        do_stuff(Path::new("test_data/equalFail"), "equalFail").unwrap();
    }

    #[test]
    fn test_not_equal() {
        do_stuff(Path::new("test_data/notEqual"), "notEqual").unwrap();
    }

    #[test]
    fn test_not_equal_fail() {
        do_stuff(Path::new("test_data/notEqualFail"), "notEqualFail").unwrap();
    }
}
