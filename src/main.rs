use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
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

// TODO Test_data should contain way more folders with a test for each
const PATH: &str = "test_data/equal";
fn main() {
    do_stuff(PATH, "main")
}

fn do_stuff(folder_path: &str, subfolder: &str) {
    let path = env::current_dir().expect("Couldn't access pwd");
    // Assert Scarb toml file
    let binding = path.join("temp").join(subfolder);
    let path_dst = binding.as_path();
    copy_dir_all(
        path.join(folder_path).as_path(),
        path_dst,
        &["cairo", "toml", "lock"],
    )
    .expect("Couldn't copy test data");

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

    fs::remove_dir_all(path_dst).expect("Error while removing temp folder");

    if failures.len() > 0 {
        println!("Found {} failing mutation(s):", failures.len());
        for (error, file, pos) in failures {
            println!("\tMutation applied {}", error);
            println!("\tFile {:?} line {:?}\n", file, pos + 1);
        }
    } else {
        println!("All mutation tests passed");
    }
}
fn change_line_content(file_path: &Path, line_number: usize, new_content: &str) -> io::Result<()> {
    // Open the file for reading
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Collect lines into a vector
    let mut lines: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    // Check if line_number is valid
    if line_number > 0 && line_number <= lines.len() {
        // Modify the content of the specified line
        lines[line_number - 1] = new_content.to_string(); // line_number is 1-based

        // Re-open the file for writing
        let mut file = File::create(file_path)?;

        // Write the modified lines back to the file
        for line in &lines {
            writeln!(file, "{}", line)?;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid line number",
        ));
    }

    Ok(())
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

fn copy_dir_all(src: &Path, dst: &Path, file_extensions: &[&str]) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dest_path = dst.join(path.file_name().unwrap());
            copy_dir_all(&path, &dest_path, file_extensions)?;
        } else {
            // Check if the file's extension matches any in file_extensions
            if let Some(ext) = path.extension() {
                if file_extensions.iter().any(|&e| ext == e) {
                    let dest_file = dst.join(path.file_name().unwrap());
                    fs::copy(&path, &dest_file)?;
                }
            }
        }
    }

    Ok(())
}

fn collect_files_with_extension(
    folder_path: &Path,
    file_extension: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    // Read the directory
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a file and its extension matches the desired one
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == file_extension {
                    files.push(path);
                }
            }
        } else if path.is_dir() {
            // Recursively collect files from subdirectories
            let mut subfolder_files = collect_files_with_extension(&path, file_extension)?;
            files.append(&mut subfolder_files);
        }
    }

    Ok(files)
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
