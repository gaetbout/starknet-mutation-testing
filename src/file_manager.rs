use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

pub fn collect_files_with_extension(
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

pub fn copy_dir_all(src: &Path, dst: &Path, file_extensions: &[&str]) -> io::Result<()> {
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

pub fn change_line_content(
    file_path: &Path,
    line_number: usize,
    new_content: &str,
) -> io::Result<()> {
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