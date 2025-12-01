use std::process::Command;

fn main() {
    // Only provide Aeron info if rusteron feature is enabled
    if cfg!(feature = "rusteron") {
        check_aeron_availability();
    }
}

fn check_aeron_availability() {
    // Check if aeronmd is available in PATH
    let aeronmd_output = Command::new("which").arg("aeronmd").output().ok();

    if let Some(output) = aeronmd_output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!(
                "cargo:warning=Aeron media driver found in PATH at: {}",
                path
            );

            // Derive library path from binary path
            // e.g., /path/to/aeron/cppbuild/Release/binaries/aeronmd
            //    -> /path/to/aeron/cppbuild/Release/lib
            if let Some(parent) = std::path::Path::new(&path).parent() {
                if let Some(release) = parent.parent() {
                    let lib_path = release.join("lib");
                    if lib_path.exists() {
                        let lib_path_str = lib_path.to_string_lossy();
                        println!("cargo:rustc-link-search=native={}", lib_path_str);
                        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path_str);
                        println!("cargo:warning=Added Aeron library path: {}", lib_path_str);
                        return;
                    }
                }
            }
        }
    }

    println!("cargo:warning=Aeron media driver not found in PATH");
    println!("cargo:warning=Integration tests will require manual media driver setup");
    println!("cargo:warning=See README.md or openspec/integration-test.md for instructions");
}
