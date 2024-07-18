use std::{
    env, fs,
    fs::File,
    io,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

const PATH: &str = "test_data";
fn main() {
    let path = env::current_dir().expect("Couldn't access pwd");
    // Assert Scarb toml file
    let binding = path.join("temp");
    let path_dst = binding.as_path();
    copy_dir_all(path.join(PATH).as_path(), path_dst).expect("Couldn't copy test data");

    // First ensure all tests pass
    // Run the scarb test command
    run_tests(path_dst);

    let files = collect_files_with_extension(path_dst, "cairo").expect("Couldn't collect files");

    // Print the collected files
    for file in &files {
        println!("File {}", file.display());
        // Read the content of the file into a string
        let content = fs::read_to_string(&file).expect("Error while reading the file");
        // Look for mutation
        let mut mutations = Vec::new();
        for (pos, line) in content.lines().into_iter().enumerate() {
            if line.contains("==") {
                mutations.push((pos, line.replace("==", "!=")));
            }
        }
        // Mutate the file
        for (pos, line) in mutations {
            change_line_content(&file, pos + 1, &line).expect("Error while changing content");
        }

        run_tests(path_dst);
    }

    // TODO Cleanup temp folder
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

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir(dst)?;
    }
    for entry in fs::read_dir(src)? {
        // TODO ignore the target folder
        // TODO ignore .gitignore (basically just copy cairo, toml and lock file)
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
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

fn run_tests(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("test")
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command failed with error: {}", stderr);
    }
    output.status.success()
}
