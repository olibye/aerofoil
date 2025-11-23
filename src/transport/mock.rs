///! Mock implementations of transport traits for testing.
///!
///! These mocks provide in-memory implementations that don't require an Aeron
///! media driver, making tests fast, deterministic, and easy to set up.

use super::{AeronPublisher, AeronSubscriber, ClaimBuffer, FragmentBuffer, FragmentHeader, TransportError};
use std::collections::VecDeque;

/// Mock publisher for testing.
///
/// This publisher stores all published messages in memory, allowing tests to
/// inspect what was published. It simulates back-pressure when configured.
///
/// # Example
///
/// ```
/// use aerofoil::transport::{MockPublisher, AeronPublisher};
///
/// let mut publisher = MockPublisher::new();
/// publisher.offer(b"hello").unwrap();
/// publisher.offer(b"world").unwrap();
///
/// let messages = publisher.published_messages();
/// assert_eq!(messages.len(), 2);
/// assert_eq!(messages[0], b"hello");
/// assert_eq!(messages[1], b"world");
/// ```
#[derive(Debug, Default)]
pub struct MockPublisher {
    messages: Vec<Vec<u8>>,
    next_position: i64,
    simulate_back_pressure: bool,
    back_pressure_threshold: Option<usize>,
}

impl MockPublisher {
    /// Creates a new mock publisher.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_position: 0,
            simulate_back_pressure: false,
            back_pressure_threshold: None,
        }
    }

    /// Returns all published messages.
    pub fn published_messages(&self) -> &[Vec<u8>] {
        &self.messages
    }

    /// Clears all published messages.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Simulates back-pressure on the next publish attempt.
    ///
    /// After calling this, the next `offer` or `try_claim` will return
    /// `TransportError::BackPressure`. Subsequent calls will succeed.
    pub fn simulate_back_pressure_once(&mut self) {
        self.simulate_back_pressure = true;
    }

    /// Sets a threshold for automatic back-pressure simulation.
    ///
    /// When the number of published messages reaches this threshold,
    /// further publish attempts will fail with back-pressure until
    /// messages are cleared.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Maximum number of messages before back-pressure
    pub fn set_back_pressure_threshold(&mut self, threshold: usize) {
        self.back_pressure_threshold = Some(threshold);
    }

    /// Removes the back-pressure threshold.
    pub fn clear_back_pressure_threshold(&mut self) {
        self.back_pressure_threshold = None;
    }

    fn check_back_pressure(&mut self) -> Result<(), TransportError> {
        // Check one-time back-pressure simulation
        if self.simulate_back_pressure {
            self.simulate_back_pressure = false;
            return Err(TransportError::BackPressure);
        }

        // Check threshold-based back-pressure
        if let Some(threshold) = self.back_pressure_threshold {
            if self.messages.len() >= threshold {
                return Err(TransportError::BackPressure);
            }
        }

        Ok(())
    }
}

impl AeronPublisher for MockPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        self.check_back_pressure()?;

        let position = self.next_position;
        self.next_position += buffer.len() as i64;
        self.messages.push(buffer.to_vec());
        Ok(position)
    }

    fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
        self.check_back_pressure()?;

        let position = self.next_position;
        self.next_position += length as i64;

        // For mock, we allocate a buffer that will be captured when dropped
        // In real implementation, this would be a view into Aeron's buffer
        let buffer = vec![0u8; length];
        self.messages.push(buffer);

        // Return a mutable reference to the last pushed buffer
        let buffer_ref = self.messages.last_mut().unwrap();
        Ok(ClaimBuffer::new(buffer_ref, position))
    }
}

/// Mock subscriber for testing.
///
/// This subscriber allows tests to inject messages and verify they are
/// received correctly via the poll interface.
///
/// # Example
///
/// ```
/// use aerofoil::transport::{MockSubscriber, AeronSubscriber};
///
/// let mut subscriber = MockSubscriber::new();
/// subscriber.inject_message(b"hello".to_vec());
/// subscriber.inject_message(b"world".to_vec());
///
/// let mut received = Vec::new();
/// subscriber.poll(|fragment| {
///     received.push(fragment.as_ref().to_vec());
///     Ok(())
/// }).unwrap();
///
/// assert_eq!(received.len(), 2);
/// assert_eq!(received[0], b"hello");
/// assert_eq!(received[1], b"world");
/// ```
#[derive(Debug, Default)]
pub struct MockSubscriber {
    messages: VecDeque<Vec<u8>>,
    next_position: i64,
    session_id: i32,
    stream_id: i32,
}

impl MockSubscriber {
    /// Creates a new mock subscriber.
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            next_position: 0,
            session_id: 1,
            stream_id: 1,
        }
    }

    /// Injects a message to be received on the next poll.
    ///
    /// Messages are queued in FIFO order.
    pub fn inject_message(&mut self, data: Vec<u8>) {
        self.messages.push_back(data);
    }

    /// Injects multiple messages at once.
    pub fn inject_messages(&mut self, messages: Vec<Vec<u8>>) {
        for msg in messages {
            self.inject_message(msg);
        }
    }

    /// Returns the number of messages waiting to be polled.
    pub fn pending_message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clears all pending messages.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Sets the session ID for generated fragment headers.
    pub fn set_session_id(&mut self, session_id: i32) {
        self.session_id = session_id;
    }

    /// Sets the stream ID for generated fragment headers.
    pub fn set_stream_id(&mut self, stream_id: i32) {
        self.stream_id = stream_id;
    }
}

impl AeronSubscriber for MockSubscriber {
    fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
    {
        let mut count = 0;

        // Process all pending messages
        while let Some(message) = self.messages.pop_front() {
            let position = self.next_position;
            self.next_position += message.len() as i64;

            let header = FragmentHeader {
                position,
                session_id: self.session_id,
                stream_id: self.stream_id,
            };

            let fragment = FragmentBuffer::new(&message, header);

            // Invoke handler - if it returns error, stop processing
            handler(&fragment)?;

            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_publisher_offer() {
        let mut publisher = MockPublisher::new();

        let pos1 = publisher.offer(b"hello").unwrap();
        let pos2 = publisher.offer(b"world").unwrap();

        assert_eq!(pos1, 0);
        assert_eq!(pos2, 5);

        let messages = publisher.published_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], b"hello");
        assert_eq!(messages[1], b"world");
    }

    #[test]
    fn test_mock_publisher_try_claim() {
        let mut publisher = MockPublisher::new();

        let mut claim = publisher.try_claim(10).unwrap();
        claim[0..5].copy_from_slice(b"hello");
        drop(claim);

        let messages = publisher.published_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(&messages[0][0..5], b"hello");
    }

    #[test]
    fn test_mock_publisher_back_pressure_once() {
        let mut publisher = MockPublisher::new();
        publisher.simulate_back_pressure_once();

        let result = publisher.offer(b"test");
        assert!(matches!(result, Err(TransportError::BackPressure)));

        // Next offer should succeed
        let result = publisher.offer(b"test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_publisher_back_pressure_threshold() {
        let mut publisher = MockPublisher::new();
        publisher.set_back_pressure_threshold(2);

        publisher.offer(b"msg1").unwrap();
        publisher.offer(b"msg2").unwrap();

        // Third message should fail
        let result = publisher.offer(b"msg3");
        assert!(matches!(result, Err(TransportError::BackPressure)));

        // Clear and try again
        publisher.clear();
        let result = publisher.offer(b"msg3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_subscriber_poll() {
        let mut subscriber = MockSubscriber::new();
        subscriber.inject_message(b"hello".to_vec());
        subscriber.inject_message(b"world".to_vec());

        let mut received = Vec::new();
        let count = subscriber
            .poll(|fragment| {
                received.push(fragment.as_ref().to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(count, 2);
        assert_eq!(received.len(), 2);
        assert_eq!(received[0], b"hello");
        assert_eq!(received[1], b"world");
    }

    #[test]
    fn test_mock_subscriber_handler_error() {
        let mut subscriber = MockSubscriber::new();
        subscriber.inject_message(b"msg1".to_vec());
        subscriber.inject_message(b"msg2".to_vec());

        let mut count = 0;
        let result = subscriber.poll(|_fragment| {
            count += 1;
            if count == 1 {
                Ok(())
            } else {
                Err(TransportError::IoError("test error".to_string()))
            }
        });

        assert!(result.is_err());
        assert_eq!(count, 2); // Handler was called twice
    }

    #[test]
    fn test_mock_subscriber_empty_poll() {
        let mut subscriber = MockSubscriber::new();

        let count = subscriber.poll(|_fragment| Ok(())).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_mock_subscriber_fragment_metadata() {
        let mut subscriber = MockSubscriber::new();
        subscriber.set_session_id(42);
        subscriber.set_stream_id(100);
        subscriber.inject_message(b"test".to_vec());

        subscriber
            .poll(|fragment| {
                let header = fragment.header();
                assert_eq!(header.session_id, 42);
                assert_eq!(header.stream_id, 100);
                assert_eq!(header.position, 0);
                Ok(())
            })
            .unwrap();
    }
}
