use crate::{
    cli::print_result,
    file_manager::{change_line_content, collect_files_with_extension, copy_dir_all},
    test_runner::{can_build, tests_successful},
};
use rayon::prelude::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
enum MutationType {
    Equal,
    NotEqual,
    // TODO Some can lead to multiple modifications
    // GreaterThan, // e.g > => >= or <
    // GreaterThanOrEqual,
    // LessThan,
    // LessThanOrEqual,
}

impl MutationType {
    fn as_str(&self) -> &str {
        match self {
            MutationType::Equal => "==",
            MutationType::NotEqual => "!=",
        }
    }

    fn others(&self, file_name: PathBuf, line: String, pos: usize) -> Vec<Mutation> {
        if !line.contains(self.as_str()) {
            return vec![];
        }

        match self {
            MutationType::Equal => vec![Mutation {
                from: self.clone(),
                to: MutationType::NotEqual,
                file_name,
                line,
                pos,
            }],
            MutationType::NotEqual => vec![Mutation {
                from: self.clone(),
                to: MutationType::Equal,
                file_name,
                line,
                pos,
            }],
        }
    }
}

#[derive(Debug)]
pub struct Mutation {
    from: MutationType,
    to: MutationType,
    file_name: PathBuf,
    line: String,
    pos: usize,
}

#[derive(Debug)]
pub enum MutationResult {
    Success(Mutation),
    BuildFailure(Mutation),
    Failure(Mutation),
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
    if !tests_successful(path_src) {
        return Err("Tests aren't passing");
    }

    let path_src = fs::canonicalize(&path_src).expect("Couldn't canonicalize path");
    find_and_test_mutations(&path_src.as_path(), "cli")
}

// TODO There must be a better way to return success or failure
fn find_and_test_mutations(path_src: &Path, subfolder: &str) -> Result<&'static str, &'static str> {
    let mutations: Vec<Mutation> = collect_mutations(path_src);
    if mutations.len() == 0 {
        return Ok("No mutations found");
    }

    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);

    let results = mutations
        .into_par_iter()
        .enumerate()
        .map(|(idx, mutation)| {
            let path_dst = &path_dst.join(idx.to_string());
            copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"])
                .expect("Couldn't copy test data");

            // Mutation from as fn
            let new_line = mutation
                .line
                .replace(mutation.from.as_str(), mutation.to.as_str());

            let file_dst = path_dst.join(mutation.file_name.clone());
            change_line_content(&file_dst, mutation.pos + 1, &new_line)
                .expect("Error applying mutation");

            if !can_build(path_dst) {
                MutationResult::BuildFailure(mutation)
            } else if tests_successful(path_dst) {
                MutationResult::Failure(mutation)
            } else {
                MutationResult::Success(mutation)
            }
        })
        .collect();
    fs::remove_dir_all(path_dst).expect("Error while removing tmp folder");
    print_result(results)
}

fn collect_mutations(path_src: &Path) -> Vec<Mutation> {
    let files = collect_files_with_extension(path_src, "cairo").expect("Couldn't collect files");

    let mutations_to_check = [MutationType::Equal, MutationType::NotEqual];
    let mut mutations: Vec<Mutation> = Vec::new();

    // TODO Transform this into a map + collect
    for file in &files {
        // Read the content of the file into a string
        let content = fs::read_to_string(&file).expect("Error while reading the file");
        let file_name = file.strip_prefix(path_src).expect("msg").to_path_buf();
        // Look for mutation
        // TODO If line is commented ==> Ignore
        for (pos, line) in content.lines().into_iter().enumerate() {
            let line = line.to_string();
            for mutation in &mutations_to_check {
                mutations.append(&mut mutation.others(file_name.clone(), line.clone(), pos));
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
