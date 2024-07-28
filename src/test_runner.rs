use std::{path::Path, process::Command};

// TODO Do a TestRunner to support other tests frameworks
pub fn can_build(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("build")
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    output.status.success()
}

pub fn tests_successful(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("test")
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    output.status.success()
}
