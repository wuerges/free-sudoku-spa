use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=scripts/public-url.mjs");
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=scripts/localization.mjs");
    let messages =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("messages.rs");
    let status = Command::new("node")
        .args(["scripts/localization.mjs", "--rust"])
        .arg(messages)
        .status()
        .expect("Node.js is required to compile translations");
    assert!(status.success(), "Translation catalog validation failed");
    for name in [
        "SUDOKU_PUBLIC_URL",
        "VERCEL_ENV",
        "VERCEL_URL",
        "VERCEL_PROJECT_PRODUCTION_URL",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    // Use the same resolver as the Trunk metadata hooks; no duplicated precedence.
    let output = Command::new("node")
        .arg("scripts/public-url.mjs")
        .output()
        .expect("Node.js is required to resolve the public build URL");
    assert!(
        output.status.success(),
        "Invalid public build URL: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let origin = String::from_utf8(output.stdout).expect("Public URL must be UTF-8");
    println!(
        "cargo:rustc-env=SUDOKU_RESOLVED_PUBLIC_URL={}",
        origin.trim()
    );
}
