use std::{path::Path, process::Command};

// TODO Do a TestRunner to support other tests frameworks
pub fn run_tests(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("test")
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    output.status.success()
}
