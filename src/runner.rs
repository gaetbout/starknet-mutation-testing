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

    let mutations_to_check: Vec<MutationType> = [
        MutationType::Equal,
        MutationType::NotEqual,
        MutationType::GreaterThan,
        MutationType::GreaterThanOrEqual,
        MutationType::LessThan,
        MutationType::LessThanOrEqual,
        MutationType::Assert,
        MutationType::IsZero,
        MutationType::IsNonZero,
        MutationType::And,
        MutationType::Or,
    ]
    .into();

    let mutations: Vec<Mutation> =
        collect_mutations(&source_folder_path, files, mutations_to_check);

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

fn collect_mutations(
    path_src: &Path,
    files: Vec<PathBuf>,
    mutations_to_check: Vec<MutationType>,
) -> Vec<Mutation> {
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
    use crate::{file_manager::collect_files_with_extension, mutant::MutationType};

    use super::{collect_mutations, test_mutations, Mutation, MutationResult};
    use rstest::rstest;
    use std::path::Path;

    #[rstest]
    #[case("equal", 1, MutationType::Equal)]
    #[case("notEqual", 1, MutationType::NotEqual)]
    #[case("greaterThan", 2, MutationType::GreaterThan)]
    #[case("greaterThanOrEqual", 2, MutationType::GreaterThanOrEqual)]
    #[case("lessThan", 2, MutationType::LessThan)]
    #[case("lessThanOrEqual", 2, MutationType::LessThanOrEqual)]
    #[case("assert", 1, MutationType::Assert)]
    #[case("isZero", 1, MutationType::IsZero)]
    #[case("isNonZero", 1, MutationType::IsNonZero)]
    #[case("and", 1, MutationType::And)]
    #[case("or", 1, MutationType::Or)]
    fn test_success(
        #[case] folder: String,
        #[case] len: usize,
        #[case] mutation_to_check: MutationType,
    ) {
        let path_src = Path::new("test_data").join(folder.clone());
        let files = collect_files_with_extension(&path_src.join("src"), "cairo")
            .expect("Couldn't collect files");
        let mutations: Vec<Mutation> = collect_mutations(&path_src, files, vec![mutation_to_check]);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert_eq!(result.len(), len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Success(_)));
        });
    }

    #[rstest]
    #[case("equalFail", 1, MutationType::Equal)]
    #[case("notEqualFail", 1, MutationType::NotEqual)]
    #[case("greaterThanFail", 2, MutationType::GreaterThan)]
    #[case("greaterThanOrEqualFail", 2, MutationType::GreaterThanOrEqual)]
    #[case("lessThanFail", 2, MutationType::LessThan)]
    #[case("lessThanOrEqualFail", 2, MutationType::LessThanOrEqual)]
    #[case("assertFail", 1, MutationType::Assert)]
    #[case("isZeroFail", 1, MutationType::IsZero)]
    #[case("isNonZeroFail", 1, MutationType::IsNonZero)]
    #[case("andFail", 1, MutationType::And)]
    #[case("orFail", 1, MutationType::Or)]
    fn test_failure(
        #[case] folder: String,
        #[case] len: usize,
        #[case] mutation_to_check: MutationType,
    ) {
        let path_src = Path::new("test_data").join(folder.clone());
        let files = collect_files_with_extension(&path_src.join("src"), "cairo")
            .expect("Couldn't collect files");
        let mutations: Vec<Mutation> = collect_mutations(&path_src, files, vec![mutation_to_check]);
        let dst = format!("tests/{}", folder);
        let result = test_mutations(path_src.as_path(), dst, mutations);
        assert_eq!(result.len(), len);
        result.iter().for_each(|r| {
            assert!(matches!(r, MutationResult::Failure(_)));
        });
    }
}
