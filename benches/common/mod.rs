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

        let driver_context = AeronDriverContext::new().map_err(|e| {
            format!(
                "Failed to create media driver context: {:?}\n\
                 Ensure Aeron C libraries are installed.",
                e
            )
        })?;

        let (stop_signal, _driver_handle) = AeronDriver::launch_embedded(driver_context, false);

        // Give the driver time to initialize
        thread::sleep(Duration::from_millis(200));

        Ok(MediaDriverGuard { stop_signal })
    }

    /// Starts a media driver for aeron-rs benchmarks.
    ///
    /// Note: aeron-rs requires an external media driver to be running.
    /// This function returns an error directing the user to start it manually.
    #[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
    pub fn start() -> Result<Self, String> {
        Err(
            "aeron-rs benchmarks require an external Aeron media driver.\n\
             Start the Java media driver with: java -cp aeron-all.jar io.aeron.driver.MediaDriver"
                .to_string(),
        )
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
