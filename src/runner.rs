use crate::{
    cli::print_result,
    file_manager::collect_files_with_extension,
    mutant::{Mutation, MutationResult, MutationType},
    test_runner::{can_build, tests_successful},
};
use rayon::prelude::*;
use std::{
    env, fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

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
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let unique = since_the_epoch.as_secs().to_string();
    let results = test_mutations(&path_src.as_path(), format!("cli/{}", unique), mutations);
    print_result(results)
}

// TODO There must be a better way to return success or failure
fn test_mutations(
    path_src: &Path,
    subfolder: String,
    mutations: Vec<Mutation>,
) -> Vec<MutationResult> {
    println!("Found {} mutations", mutations.len());
    let path_dst = &env::current_dir()
        .expect("Couldn't access pwd")
        .join("tmp")
        .join(subfolder);

    let results = mutations
        .into_iter()
        // .into_par_iter()
        .enumerate()
        .map(|(idx, mutation)| {
            let path_dst = &path_dst.join(idx.to_string());

            mutation.apply_mutation(path_src, path_dst);

            if !can_build(path_dst) {
                println!("Build failed for mutation {:?}", mutation);
                MutationResult::BuildFailure(mutation)
            } else if tests_successful(path_dst) {
                println!("Test failed for mutation {:?}", mutation);
                MutationResult::Failure(mutation)
            } else {
                println!("Test passed for mutation {:?}", mutation);
                MutationResult::Success(mutation)
            }
        })
        .collect();
    fs::remove_dir_all(path_dst).expect("Error while removing tmp folder");
    results
}

fn collect_mutations(path_src: &Path) -> Vec<Mutation> {
    let files = collect_files_with_extension(&path_src.join("src"), "cairo")
        .expect("Couldn't collect files");

    let mutations_to_check = [
        MutationType::Equal,
        MutationType::NotEqual,
        MutationType::GreaterThan,
        MutationType::GreaterThanOrEqual,
        MutationType::LessThan,
        MutationType::LessThanOrEqual,
    ];
    let mut mutations: Vec<Mutation> = Vec::new();

    // TODO Transform this into a map + collect
    for file in &files {
        // Read the content of the file into a string
        let content = fs::read_to_string(&file).expect("Error while reading the file");
        let file_name = file.strip_prefix(path_src).expect("msg").to_path_buf();
        // Look for mutation
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
    #[case("equal", 1)]
    #[case("notEqual", 1)]
    #[case("greaterThan", 2)]
    #[case("greaterThanOrEqual", 2)]
    #[case("lessThan", 2)]
    #[case("lessThanOrEqual", 2)]
    fn test_success(#[case] folder: String, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder.clone());
        let mutations: Vec<Mutation> = collect_mutations(&path_src);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert!(result.len() == len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Success(_)));
        });
    }

    #[rstest]
    #[case("equalFail", 1)]
    #[case("notEqualFail", 1)]
    #[case("greaterThanFail", 2)]
    #[case("greaterThanOrEqualFail", 2)]
    #[case("lessThanFail", 2)]
    #[case("lessThanOrEqualFail", 2)]
    fn test_failure(#[case] folder: String, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder.clone());
        let mutations: Vec<Mutation> = collect_mutations(&path_src);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert!(result.len() == len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Failure(_)));
        });
    }
}
