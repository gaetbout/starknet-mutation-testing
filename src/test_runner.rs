use std::{path::Path, process::Command};

// TODO Do a TestRunner to support other tests frameworks
pub fn can_build(path_dst: &Path) -> bool {
    let output = Command::new("scarb")
        .arg("build")
        .env("SCARB_CACHE", path_dst.as_os_str())
        .current_dir(path_dst)
        .output()
        .expect("Failed to execute command");

    // println!("{:?}", output);
    output.status.success()
}

// A bit ugly, let's change it later
pub fn tests_successful(path_dst: &Path, with_env: bool) -> bool {
    let output = if with_env {
        Command::new("scarb")
            .arg("test")
            .env("SCARB_CACHE", path_dst.as_os_str())
            .current_dir(path_dst)
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("scarb")
            .arg("test")
            .current_dir(path_dst)
            .output()
            .expect("Failed to execute command")
    };

    output.status.success()
}
