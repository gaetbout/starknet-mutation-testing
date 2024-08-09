use crate::{
    cli::print_result,
    file_manager::{collect_files_with_extension, get_tmp_dir},
    mutant::{Mutation, MutationResult, MutationType},
    test_runner::{can_build, tests_successful},
    Result,
};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn run_mutation_checks(
    source_folder_path: PathBuf,
    file_to_check: Option<PathBuf>,
) -> Result<()> {
    let files: Vec<PathBuf> = if let Some(file) = file_to_check {
        vec![file]
    } else {
        collect_files_with_extension(&source_folder_path.join("src"), "cairo")
            .expect("Couldn't collect files")
    };
    let mutations: Vec<Mutation> = collect_mutations(&source_folder_path, files);

    if mutations.len() == 0 {
        println!("No mutations found");
        return Ok(());
    }
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let unique = since_the_epoch.as_secs().to_string();
    let results = test_mutations(
        source_folder_path.as_path(),
        format!("cli/{}", unique),
        mutations,
    );
    print_result(results)
}

fn test_mutations(
    path_src: &Path,
    subfolder: String,
    mutations: Vec<Mutation>,
) -> Vec<MutationResult> {
    println!("Found {} mutations, running tests...", mutations.len());
    let path_dst = get_tmp_dir().join(subfolder);

    let resolved_mutations = AtomicUsize::new(0);
    let len = mutations.len();
    let results = mutations
        .into_par_iter()
        .enumerate()
        .map(|(idx, mutation)| {
            let path_dst = &path_dst.join(idx.to_string());

            mutation.apply_mutation(path_src, path_dst);

            let res = if !can_build(path_dst) {
                MutationResult::BuildFailure(mutation)
            } else if tests_successful(path_dst, true) {
                MutationResult::Failure(mutation)
            } else {
                MutationResult::Success(mutation)
            };

            println!("{:?}", res);

            resolved_mutations.fetch_add(1, Ordering::SeqCst);

            println!(
                "Resolved {}/{} mutations",
                resolved_mutations.load(Ordering::SeqCst),
                len
            );
            res
        })
        .collect();
    fs::remove_dir_all(path_dst).expect("Error while removing tmp folder");
    results
}

fn collect_mutations(path_src: &Path, files: Vec<PathBuf>) -> Vec<Mutation> {
    let mutations_to_check = [
        MutationType::Equal,
        MutationType::NotEqual,
        MutationType::GreaterThan,
        MutationType::GreaterThanOrEqual,
        MutationType::LessThan,
        MutationType::LessThanOrEqual,
        MutationType::Assert,
        MutationType::IsZero,
        MutationType::IsNonZero,
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
            // We consider the rest of the file as test code
            if line.contains("#[cfg(test)]") {
                break;
            }
            for mutation in &mutations_to_check {
                mutations.append(&mut mutation.others(file_name.clone(), line.clone(), pos));
            }
        }
    }
    mutations
}

#[cfg(test)]
mod tests {
    use crate::file_manager::collect_files_with_extension;

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
    #[case("assert", 1)]
    #[case("isZero", 2)]
    #[case("isNonZero", 2)]
    fn test_success(#[case] folder: String, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder.clone());
        let files = collect_files_with_extension(&path_src.join("src"), "cairo")
            .expect("Couldn't collect files");
        let mutations: Vec<Mutation> = collect_mutations(&path_src, files);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert_eq!(result.len(), len);
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
    #[case("assertFail", 1)]
    #[case("isZeroFail", 2)]
    #[case("isNonZeroFail", 2)]
    fn test_failure(#[case] folder: String, #[case] len: usize) {
        let path_src = Path::new("test_data").join(folder.clone());
        let files = collect_files_with_extension(&path_src.join("src"), "cairo")
            .expect("Couldn't collect files");
        let mutations: Vec<Mutation> = collect_mutations(&path_src, files);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert_eq!(result.len(), len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Failure(_)));
        });
    }
}
