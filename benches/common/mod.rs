//! Common benchmark utilities.
//!
//! Provides shared helpers for Aeron transport benchmarks, following the project
//! convention to factor out common code into helper functions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub mod generic;

/// RAII guard for managing Aeron media driver lifecycle in benchmarks.
///
/// The media driver is automatically started on creation and stopped on drop,
/// ensuring proper cleanup even if the benchmark panics.
pub struct MediaDriverGuard {
    stop_signal: Arc<AtomicBool>,
}

impl MediaDriverGuard {
    /// Starts an embedded Aeron media driver, or uses an external one if AERON_EXTERNAL_DRIVER=1.
    ///
    /// Both rusteron and aeron-rs use the same embedded driver from rusteron-media-driver.
    /// Set AERON_EXTERNAL_DRIVER=1 environment variable to skip embedded driver and use an
    /// external one (e.g., Java media driver).
    ///
    /// # Errors
    ///
    /// Returns an error if the media driver cannot be started.
    #[cfg(any(feature = "rusteron", feature = "aeron-rs"))]
    pub fn start() -> Result<Self, String> {
        // Check if user wants to use an external driver
        if std::env::var("AERON_EXTERNAL_DRIVER").is_ok() {
            eprintln!("Using external Aeron media driver (AERON_EXTERNAL_DRIVER is set)");
            return Ok(MediaDriverGuard {
                stop_signal: Arc::new(AtomicBool::new(false)),
            });
        }

        Self::start_embedded()
    }

    #[cfg(any(feature = "rusteron", feature = "aeron-rs"))]
    fn start_embedded() -> Result<Self, String> {
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
                 Ensure Aeron C libraries are installed, or set AERON_EXTERNAL_DRIVER=1 \
                 and run an external media driver.",
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

// ============================================================================
// Rusteron benchmark helpers
// ============================================================================

#[cfg(feature = "rusteron")]
pub mod rusteron_support {
    use super::*;
    use rusteron_client::IntoCString;

    /// Benchmark context holding the media driver and Aeron client.
    ///
    /// Use this to set up benchmarks with a single shared driver and client.
    #[allow(dead_code)]
    pub struct BenchContext {
        pub driver: MediaDriverGuard,
        pub aeron: rusteron_client::Aeron,
    }

    #[allow(dead_code)]
    impl BenchContext {
        /// Creates a new benchmark context with an embedded media driver and Aeron client.
        pub fn new() -> Option<Self> {
            let driver = match MediaDriverGuard::start() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Skipping benchmark: {}", e);
                    return None;
                }
            };

            let context =
                rusteron_client::AeronContext::new().expect("Failed to create Aeron context");
            let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
            aeron.start().expect("Failed to start Aeron");

            Some(BenchContext { driver, aeron })
        }

        /// Adds a publication and waits for it to be ready.
        pub fn add_publication(
            &self,
            stream_id: i32,
        ) -> rusteron_client::AeronPublication {
            let async_pub = self
                .aeron
                .async_add_publication(&CHANNEL.into_c_string(), stream_id)
                .expect("Failed to start publication");

            let publication = async_pub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete publication");

            thread::sleep(Duration::from_millis(100));
            publication
        }

        /// Adds a subscription and waits for it to be ready.
        pub fn add_subscription(
            &self,
            stream_id: i32,
        ) -> rusteron_client::AeronSubscription {
            let async_sub = self
                .aeron
                .async_add_subscription(
                    &CHANNEL.into_c_string(),
                    stream_id,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed to start subscription");

            let subscription = async_sub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete subscription");

            thread::sleep(Duration::from_millis(100));
            subscription
        }

        /// Adds both a publication and subscription on the same stream.
        pub fn add_pub_sub(
            &self,
            stream_id: i32,
        ) -> (rusteron_client::AeronPublication, rusteron_client::AeronSubscription) {
            let publication = self.add_publication(stream_id);
            let subscription = self.add_subscription(stream_id);
            (publication, subscription)
        }
    }
}

// ============================================================================
// Aeron-rs benchmark helpers
// ============================================================================

#[cfg(feature = "aeron-rs")]
pub mod aeron_rs_support {
    use super::*;
    use aeron_rs::aeron::Aeron;
    use aeron_rs::context::Context;
    use aeron_rs::publication::Publication;
    use aeron_rs::subscription::Subscription;
    use std::ffi::CString;
    use std::sync::{Arc, Mutex};

    /// Benchmark context holding the media driver and Aeron client.
    ///
    /// Use this to set up benchmarks with a single shared driver and client.
    #[allow(dead_code)]
    pub struct BenchContext {
        pub driver: MediaDriverGuard,
        pub aeron: Aeron,
    }

    #[allow(dead_code)]
    impl BenchContext {
        /// Creates a new benchmark context with an embedded media driver and Aeron client.
        pub fn new() -> Option<Self> {
            let driver = match MediaDriverGuard::start() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Skipping benchmark: {}", e);
                    return None;
                }
            };

            let context = Context::new();
            let aeron = match Aeron::new(context) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "Failed to connect to Aeron media driver: {:?}\n\
                         Ensure the media driver started successfully.",
                        e
                    );
                    return None;
                }
            };

            Some(BenchContext { driver, aeron })
        }

        /// Adds a publication and waits for it to be ready.
        pub fn add_publication(&mut self, stream_id: i32) -> Arc<Mutex<Publication>> {
            let channel = CString::new(CHANNEL).expect("Invalid channel");

            let registration_id = self
                .aeron
                .add_publication(channel, stream_id)
                .expect("Failed to add publication");

            // Poll until publication is ready
            let publication = loop {
                match self.aeron.find_publication(registration_id) {
                    Ok(pub_arc) => break pub_arc,
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            };

            thread::sleep(Duration::from_millis(100));
            publication
        }

        /// Adds a subscription and waits for it to be ready.
        pub fn add_subscription(&mut self, stream_id: i32) -> Arc<Mutex<Subscription>> {
            let channel = CString::new(CHANNEL).expect("Invalid channel");

            let registration_id = self
                .aeron
                .add_subscription(channel, stream_id)
                .expect("Failed to add subscription");

            // Poll until subscription is ready
            let subscription = loop {
                match self.aeron.find_subscription(registration_id) {
                    Ok(sub_arc) => break sub_arc,
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            };

            thread::sleep(Duration::from_millis(100));
            subscription
        }

        /// Adds both a publication and subscription on the same stream.
        pub fn add_pub_sub(
            &mut self,
            stream_id: i32,
        ) -> (Arc<Mutex<Publication>>, Arc<Mutex<Subscription>>) {
            let publication = self.add_publication(stream_id);
            let subscription = self.add_subscription(stream_id);
            (publication, subscription)
        }
    }
}
