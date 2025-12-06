//! Common benchmark utilities.
//!
//! Provides shared helpers for Aeron transport benchmarks, following the project
//! convention to factor out common code into helper functions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// RAII guard for managing Aeron media driver lifecycle in benchmarks.
///
/// The media driver is automatically started on creation and stopped on drop,
/// ensuring proper cleanup even if the benchmark panics.
pub struct MediaDriverGuard {
    stop_signal: Arc<AtomicBool>,
}

impl MediaDriverGuard {
    /// Starts an embedded Aeron media driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the media driver cannot be started.
    #[cfg(feature = "rusteron")]
    pub fn start() -> Result<Self, String> {
        use rusteron_media_driver::{AeronDriver, AeronDriverContext};

        // Clean up any stale Aeron state from previous runs
        // Default directory varies by platform: /dev/shm/aeron on Linux, /tmp/aeron-{user} on macOS
        if let Ok(aeron_dir) = std::env::var("AERON_DIR") {
            let _ = std::fs::remove_dir_all(&aeron_dir);
        } else {
            // Try common default locations
            let _ = std::fs::remove_dir_all("/dev/shm/aeron");
            if let Ok(user) = std::env::var("USER") {
                let _ = std::fs::remove_dir_all(format!("/tmp/aeron-{}", user));
            }
        }

        let driver_context = AeronDriverContext::new().map_err(|e| {
            format!(
                "Failed to create media driver context: {:?}\n\
                 Ensure Aeron C libraries are installed.",
                e
            )
        })?;

        // Increase timeouts for benchmarks to avoid service interval exceeded errors
        // Default is 10 seconds which is too short for longer benchmark runs
        driver_context
            .set_driver_timeout_ms(60_000) // 60 seconds driver timeout
            .map_err(|e| format!("Failed to set driver timeout: {:?}", e))?;
        driver_context
            .set_client_liveness_timeout_ns(60_000_000_000) // 60 seconds client liveness
            .map_err(|e| format!("Failed to set client liveness timeout: {:?}", e))?;

        let (stop_signal, _driver_handle) = AeronDriver::launch_embedded(driver_context, false);

        // Give the driver time to initialize
        thread::sleep(Duration::from_millis(200));

        Ok(MediaDriverGuard { stop_signal })
    }

    /// Starts a media driver for aeron-rs benchmarks.
    ///
    /// Note: aeron-rs requires an external media driver to be running.
    /// This returns a dummy guard that assumes the driver is already running.
    /// If no driver is available, connection will fail when creating publications.
    #[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
    pub fn start() -> Result<Self, String> {
        // aeron-rs requires an external media driver - assume it's running
        // Connection errors will surface when trying to create publications/subscriptions
        Ok(MediaDriverGuard {
            stop_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    #[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
    pub fn start() -> Result<Self, String> {
        Err("No Aeron backend feature enabled. Enable 'rusteron' or 'aeron-rs'.".to_string())
    }
}

impl Drop for MediaDriverGuard {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(100));
    }
}

/// Default IPC channel for benchmarks.
pub const CHANNEL: &str = "aeron:ipc";

/// Message size variants for benchmarks.
#[derive(Debug, Clone, Copy)]
pub enum MessageSize {
    /// Small message (64 bytes) - typical for simple numeric data
    Small,
    /// Medium message (1024 bytes) - typical for structured messages
    Medium,
    /// Large message (8192 bytes) - typical for batch operations
    Large,
}

impl MessageSize {
    /// Returns the size in bytes.
    pub fn bytes(&self) -> usize {
        match self {
            MessageSize::Small => 64,
            MessageSize::Medium => 1024,
            MessageSize::Large => 8192,
        }
    }

    /// Creates a buffer of the specified size filled with test data.
    pub fn create_buffer(&self) -> Vec<u8> {
        vec![0xABu8; self.bytes()]
    }

    /// Returns a display name for the message size.
    pub fn name(&self) -> &'static str {
        match self {
            MessageSize::Small => "64B",
            MessageSize::Medium => "1KB",
            MessageSize::Large => "8KB",
        }
    }
}
