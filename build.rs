use std::process::Command;

fn main() {
    // Only provide Aeron info if rusteron feature is enabled
    if cfg!(feature = "rusteron") {
        check_aeron_availability();
    }
}

fn check_aeron_availability() {
    // Check if aeronmd is available in PATH
    let aeronmd_found = Command::new("which")
        .arg("aeronmd")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if aeronmd_found {
        println!("cargo:warning=Aeron media driver found in PATH");
    } else {
        println!("cargo:warning=Aeron media driver not found in PATH");
        println!("cargo:warning=Integration tests will require manual media driver setup");
        println!("cargo:warning=See README.md or openspec/integration-test.md for instructions");
    }

    // Check for common macOS installation locations
    let common_paths = [
        "/usr/local/bin/aeronmd",
        "/opt/homebrew/bin/aeronmd",
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            println!("cargo:rustc-env=AERON_MEDIA_DRIVER_PATH={}", path);
            println!("cargo:warning=Found media driver at: {}", path);
            return;
        }
    }
}
