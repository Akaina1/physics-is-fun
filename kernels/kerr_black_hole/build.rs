// Build script to capture provenance information at compile time

use std::process::Command;

fn main() {
    // Capture git SHA (short form)
    let git_sha = Command::new("git")
        .args(&["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    // Capture Rust compiler version
    let rustc_version = std::env::var("RUSTC_VERSION")
        .unwrap_or_else(|_| {
            Command::new("rustc")
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });
    
    // Capture build timestamp (ISO 8601 format)
    let build_timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    
    // Emit as environment variables for the compiled binary
    println!("cargo:rustc-env=BUILD_GIT_SHA={}", git_sha);
    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={}", rustc_version);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
    
    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/heads");
}

