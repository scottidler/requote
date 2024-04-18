// build.rs
//
use std::env;
use std::fs::{File, read_to_string, write};
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn git_describe_value() -> String {
    // First, check if GIT_DESCRIBE env var is set and use it if so
    if let Ok(value) = env::var("GIT_DESCRIBE") {
        println!("Using GIT_DESCRIBE from environment: {}", value);
        return value;
    }

    // Fallback to using git command
    match Command::new("git")
        .args(&["describe", "--tags", "--always"])
        .output() {
        Ok(output) if output.status.success() => {
            let git_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("GIT_DESCRIBE from 'git' command: {}", git_output);
            git_output
        },
        Ok(output) => {
            let err_output = String::from_utf8_lossy(&output.stderr).trim().to_string();
            println!("Failed to run 'git describe', fallback to 'unknown'. Error: {}", err_output);
            "unknown".to_string()
        },
        Err(e) => {
            println!("Error executing 'git': {}. Falling back to 'unknown'.", e);
            "unknown".to_string()
        }
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("git_describe.rs");
    println!("Output path for git_describe.rs: {}", dest_path.display());

    let current_version = git_describe_value();

    // Compare with existing version, if any, to determine if we need to update
    let old_version = read_to_string(&dest_path).unwrap_or_default();
    if old_version.contains(&current_version) {
        println!("Version unchanged ({}), skipping update.", current_version);
        return;
    }

    // Write the current version to git_describe.rs
    match File::create(&dest_path).and_then(|mut file| {
        writeln!(file, "pub const GIT_DESCRIBE: &str = \"{}\";", current_version)
    }) {
        Ok(_) => println!("Successfully updated git_describe.rs."),
        Err(e) => panic!("Failed to write to {}: {}", dest_path.display(), e),
    }

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-env-changed=GIT_DESCRIBE");
}
