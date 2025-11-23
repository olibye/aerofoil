//! Tests demonstrating trait usage without external Aeron dependencies.
//!
//! Note: mockall's automock has limitations with complex lifetimes and closures,
//! so these tests demonstrate manual test implementations.

#[cfg(test)]
mod trait_tests {
    use crate::transport::{AeronPublisher, AeronSubscriber, ClaimBuffer, FragmentBuffer, FragmentHeader, TransportError};
    use std::collections::VecDeque;

    /// Simple test publisher for demonstration
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

        fn messages(&self) -> &[Vec<u8>] {
            &self.messages
        }
    }

    impl AeronPublisher for TestPublisher {
        fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
            let pos = self.next_position;
            self.next_position += buffer.len() as i64;
            self.messages.push(buffer.to_vec());
            Ok(pos)
        }

        fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
            let pos = self.next_position;
            self.next_position += length as i64;
            self.messages.push(vec![0u8; length]);
            let buf = self.messages.last_mut().unwrap();
            Ok(ClaimBuffer::new(buf, pos))
        }
    }

    /// Simple test subscriber for demonstration
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
            while let Some(msg) = self.messages.pop_front() {
                let pos = self.next_position;
                self.next_position += msg.len() as i64;

                let header = FragmentHeader {
                    position: pos,
                    session_id: 1,
                    stream_id: 1,
                };
                let fragment = FragmentBuffer::new(&msg, header);
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
