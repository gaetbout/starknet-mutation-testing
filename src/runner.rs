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
    GreaterThan, // TODO Should I just add all 3 other mutations for each greaterThan, greaterThanOrEqual, lessThan, lessThanOrEqual?
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl MutationType {
    fn as_str(&self) -> &str {
        match self {
            MutationType::Equal => "==",
            MutationType::NotEqual => "!=",
            MutationType::GreaterThan => " > ", // TODO fix: should ignore if it is "->"
            MutationType::GreaterThanOrEqual => ">=",
            MutationType::LessThan => " < ",
            MutationType::LessThanOrEqual => "<=",
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
            MutationType::GreaterThan => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::GreaterThanOrEqual,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::LessThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
            MutationType::GreaterThanOrEqual => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::Equal,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::GreaterThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
            MutationType::LessThan => {
                vec![]
            }
            MutationType::LessThanOrEqual => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::Equal,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::LessThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
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

impl Mutation {
    fn apply_mutation(&self, path_src: &Path, path_dst: &Path) {
        copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"])
            .expect("Couldn't copy test data");

        // Mutation from as fn
        let new_line = self.line.replace(self.from.as_str(), self.to.as_str());

        let file_dst = path_dst.join(self.file_name.clone());
        change_line_content(&file_dst, self.pos + 1, &new_line).expect("Error applying mutation");
    }
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
    let mutations: Vec<Mutation> = collect_mutations(&path_src.as_path());
    if mutations.len() == 0 {
        return Ok("No mutations found");
    }
    let results = test_mutations(&path_src.as_path(), "cli", mutations);
    print_result(results)
}

// TODO There must be a better way to return success or failure
fn test_mutations(
    path_src: &Path,
    subfolder: &str,
    mutations: Vec<Mutation>,
) -> Vec<MutationResult> {
    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);

    let results = mutations
        .into_par_iter()
        .enumerate()
        .map(|(idx, mutation)| {
            let path_dst = &path_dst.join(idx.to_string());

            mutation.apply_mutation(path_src, path_dst);

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
    results
}

fn collect_mutations(path_src: &Path) -> Vec<Mutation> {
    let files = collect_files_with_extension(path_src, "cairo").expect("Couldn't collect files");

    let mutations_to_check = [
        MutationType::Equal,
        MutationType::NotEqual,
        MutationType::GreaterThan,
        MutationType::GreaterThanOrEqual,
        MutationType::LessThanOrEqual,
    ];
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
    use super::{collect_mutations, test_mutations, Mutation, MutationResult};
    use rstest::rstest;
    use std::path::Path;

    #[rstest]
    #[case("equal", 2)]
    #[case("notEqual", 2)]
    #[case("greaterThen", 4)]
    #[case("greaterThenOrEqual", 4)]
    #[case("lessThenOrEqual", 4)]
    fn test_success(#[case] folder: &str, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder);
        let mutations: Vec<Mutation> = collect_mutations(&path_src);
        let result = test_mutations(path_src.as_path(), folder, mutations);
        assert!(result.len() == len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Success(_)));
        });
    }

    #[rstest]
    #[case("equalFail", 2)]
    #[case("notEqualFail", 2)]
    #[case("greaterThenFail", 4)]
    #[case("greaterThenOrEqualFail", 4)]
    #[case("lessThenOrEqualFail", 4)]
    fn test_failure(#[case] folder: &str, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder);
        let mutations: Vec<Mutation> = collect_mutations(&path_src);
        let result = test_mutations(path_src.as_path(), folder, mutations);
        assert!(result.len() == len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Failure(_)));
        });
    }
}
