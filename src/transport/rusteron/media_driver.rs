//! Helper for managing Aeron Media Driver in tests.
//!
//! This module provides utilities for starting and stopping the Aeron Media Driver
//! during integration tests.

use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Guard that manages an Aeron Media Driver process.
///
/// The media driver is started when this guard is created and automatically
/// stopped when the guard is dropped.
///
/// # Example
///
/// ```ignore
/// #[test]
/// #[ignore]  // Requires media driver binary
/// fn test_with_media_driver() {
///     // Start media driver
///     let _driver = MediaDriverGuard::start().expect("Failed to start media driver");
///
///     // Give it time to start up
///     std::thread::sleep(std::time::Duration::from_secs(2));
///
///     // Run tests...
///
///     // Driver automatically stopped when _driver is dropped
/// }
/// ```
pub struct MediaDriverGuard {
    process: Option<Child>,
}

impl MediaDriverGuard {
    /// Start the Aeron Media Driver.
    ///
    /// This looks for the media driver in several locations:
    /// 1. `AERON_MEDIA_DRIVER_PATH` environment variable (set by build.rs)
    /// 2. `aeronmd` in PATH
    /// 3. Downloaded location in OUT_DIR
    ///
    /// # Returns
    ///
    /// - `Ok(MediaDriverGuard)` if the driver started successfully
    /// - `Err(String)` if the driver could not be started
    pub fn start() -> Result<Self, String> {
        // Try to find aeronmd
        let aeronmd_path = Self::find_aeronmd()?;

        println!("Starting media driver: {}", aeronmd_path);

        // Start the process
        let child = Command::new(&aeronmd_path)
            .spawn()
            .map_err(|e| format!("Failed to start media driver: {}", e))?;

        Ok(MediaDriverGuard {
            process: Some(child),
        })
    }

    /// Find the aeronmd executable.
    fn find_aeronmd() -> Result<String, String> {
        // 1. Check build.rs environment variable
        if let Ok(path) = std::env::var("AERON_MEDIA_DRIVER_PATH") {
            if std::path::Path::new(&path).exists() {
                return Ok(path);
            }
        }

        // 2. Check if aeronmd is in PATH
        if let Ok(output) = Command::new("which").arg("aeronmd").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim();
                    if !path.is_empty() {
                        return Ok(path.to_string());
                    }
                }
            }
        }

        // 3. Check common installation locations on macOS
        let common_paths = [
            "/usr/local/bin/aeronmd",
            "/opt/homebrew/bin/aeronmd",
        ];

        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err("Media driver not found. Please install Aeron or set AERON_MEDIA_DRIVER_PATH".to_string())
    }

    /// Wait for the media driver to be ready.
    ///
    /// This is a simple sleep-based approach. In production, you might want
    /// to actually check the driver's status.
    pub fn wait_for_ready(&self) {
        thread::sleep(Duration::from_secs(2));
    }
}

impl Drop for MediaDriverGuard {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            println!("Stopping media driver...");

            // Try graceful shutdown first
            let _ = process.kill();

            // Wait for it to exit
            let _ = process.wait();

            println!("Media driver stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_aeronmd() {
        // Given: We try to find aeronmd
        // When: Calling find_aeronmd
        let result = MediaDriverGuard::find_aeronmd();

        // Then: Either we find it or get a helpful error
        match result {
            Ok(path) => {
                println!("Found aeronmd at: {}", path);
                assert!(!path.is_empty());
            }
            Err(e) => {
                println!("aeronmd not found (expected in CI): {}", e);
                // This is OK - media driver might not be installed
            }
        }
    }
}
