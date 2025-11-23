//! Tests demonstrating trait usage without external Aeron dependencies.
//!
//! # Design Decision: Manual Test Implementations Instead of Mockall
//!
//! These tests use hand-written implementations of `AeronPublisher` and `AeronSubscriber`
//! rather than mockall's `#[automock]` attribute because:
//!
//! ## Why Not Mockall?
//!
//! 1. **Lifetime limitations**: `try_claim<'a>(&'a mut self) -> ClaimBuffer<'a>`
//!    uses explicit lifetime parameters that mockall cannot generate correctly
//!
//! 2. **Generic closure limitations**: `poll<F: FnMut(&FragmentBuffer)>` uses a
//!    generic closure parameter that mockall's procedural macro cannot handle
//!
//! 3. **Compilation errors**: Attempting to use `#[automock]` results in:
//!    - "the parameter type F may not live long enough" errors
//!    - Missing trait method implementations in generated mocks
//!
//! ## Benefits of Manual Implementations
//!
//! - **Simplicity**: Each test impl is ~20 lines of straightforward code
//! - **Full control**: Can implement exactly the behavior needed for each test
//! - **No dependencies**: Avoids mockall as a dev-dependency
//! - **Educational**: Shows users exactly how to implement traits for testing
//! - **Flexibility**: Can add custom inspection methods (like `messages()`)
//!
//! ## Pattern for Users
//!
//! Users should follow this pattern for their own tests:
//!
//! 1. Create simple structs to hold test state (Vec for messages, etc.)
//! 2. Implement the minimal trait methods needed for the test
//! 3. Add helper methods for test assertions (e.g., `messages()` accessor)
//!
//! These test implementations serve as examples users can copy and adapt.

#[cfg(test)]
mod trait_tests {
    use crate::transport::{AeronPublisher, AeronSubscriber, ClaimBuffer, FragmentBuffer, FragmentHeader, TransportError};
    use std::collections::VecDeque;

    /// Simple test publisher for demonstration.
    ///
    /// Stores published messages in a Vec for later inspection.
    /// Tracks position to simulate Aeron's stream position concept.
    struct TestPublisher {
        messages: Vec<Vec<u8>>,
        next_position: i64,
    }

    impl TestPublisher {
        fn new() -> Self {
            Self {
                messages: Vec::new(),
                next_position: 0,
            }
        }

        /// Accessor for inspecting published messages in tests.
        fn messages(&self) -> &[Vec<u8>] {
            &self.messages
        }
    }

    impl AeronPublisher for TestPublisher {
        fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
            // Copy the message into our storage
            let pos = self.next_position;
            self.next_position += buffer.len() as i64;
            self.messages.push(buffer.to_vec());
            Ok(pos)
        }

        fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
            // Pre-allocate buffer space and return a mutable reference to it
            let pos = self.next_position;
            self.next_position += length as i64;
            self.messages.push(vec![0u8; length]);

            // Get mutable reference to the buffer we just pushed
            let buf = self.messages.last_mut().unwrap();

            // Return ClaimBuffer wrapping our storage
            // Note: The lifetime 'a ties the buffer to &'a mut self, ensuring
            // the buffer can't outlive this publisher instance
            Ok(ClaimBuffer::new(buf, pos))
        }
    }

    /// Simple test subscriber for demonstration.
    ///
    /// Uses VecDeque to simulate a message queue that can be polled.
    /// Tracks position to simulate Aeron's stream position concept.
    struct TestSubscriber {
        messages: VecDeque<Vec<u8>>,
        next_position: i64,
    }

    impl TestSubscriber {
        fn new() -> Self {
            Self {
                messages: VecDeque::new(),
                next_position: 0,
            }
        }

        /// Injects a message for the subscriber to deliver on next poll.
        /// This simulates receiving messages from an Aeron channel.
        fn inject(&mut self, data: Vec<u8>) {
            self.messages.push_back(data);
        }
    }

    impl AeronSubscriber for TestSubscriber {
        fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
        where
            F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            let mut count = 0;

            // Drain all available messages, calling handler for each
            while let Some(msg) = self.messages.pop_front() {
                let pos = self.next_position;
                self.next_position += msg.len() as i64;

                // Create header metadata for this fragment
                let header = FragmentHeader {
                    position: pos,
                    session_id: 1,
                    stream_id: 1,
                };

                // Wrap message data in FragmentBuffer
                // Note: The lifetime here is bound to the scope of this loop iteration,
                // ensuring the buffer can't escape the handler callback
                let fragment = FragmentBuffer::new(&msg, header);

                // Invoke the user's handler. If it returns an error, stop polling
                // and propagate the error (standard Aeron behavior)
                handler(&fragment)?;
                count += 1;
            }

            Ok(count)
        }
    }

    #[test]
    fn test_publisher_offer() {
        let mut publisher = TestPublisher::new();
        let pos = publisher.offer(b"hello").unwrap();
        assert_eq!(pos, 0);
        assert_eq!(publisher.messages()[0], b"hello");
    }

    #[test]
    fn test_publisher_try_claim() {
        let mut publisher = TestPublisher::new();
        let mut claim = publisher.try_claim(10).unwrap();
        claim[0..4].copy_from_slice(b"test");
        drop(claim);
        assert_eq!(&publisher.messages()[0][0..4], b"test");
    }

    #[test]
    fn test_subscriber_poll() {
        let mut subscriber = TestSubscriber::new();
        subscriber.inject(b"msg1".to_vec());
        subscriber.inject(b"msg2".to_vec());

        let mut received = Vec::new();
        let count = subscriber.poll(|frag| {
            received.push(frag.as_ref().to_vec());
            Ok(())
        }).unwrap();

        assert_eq!(count, 2);
        assert_eq!(received[0], b"msg1");
        assert_eq!(received[1], b"msg2");
    }

    #[test]
    fn test_generic_function() {
        fn send_heartbeat<P: AeronPublisher>(publisher: &mut P) -> Result<i64, TransportError> {
            publisher.offer(b"HEARTBEAT")
        }

        let mut publisher = TestPublisher::new();
        let pos = send_heartbeat(&mut publisher).unwrap();
        assert_eq!(pos, 0);
        assert_eq!(publisher.messages()[0], b"HEARTBEAT");
    }

    #[test]
    fn test_error_propagation() {
        struct ErrorPublisher;

        impl AeronPublisher for ErrorPublisher {
            fn offer(&mut self, _buffer: &[u8]) -> Result<i64, TransportError> {
                Err(TransportError::BackPressure)
            }

            fn try_claim<'a>(&'a mut self, _length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
                Err(TransportError::NotConnected)
            }
        }

        let mut publisher = ErrorPublisher;
        assert!(matches!(publisher.offer(b"test"), Err(TransportError::BackPressure)));
        assert!(matches!(publisher.try_claim(10), Err(TransportError::NotConnected)));
    }
}
