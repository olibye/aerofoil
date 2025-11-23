///! Integration tests demonstrating generic code using transport traits.

#[cfg(test)]
mod integration_tests {
    use crate::transport::{AeronPublisher, AeronSubscriber, MockPublisher, MockSubscriber, TransportError};

    // Generic function that works with any publisher
    fn send_heartbeat<P: AeronPublisher>(publisher: &mut P) -> Result<i64, TransportError> {
        publisher.offer(b"HEARTBEAT")
    }

    // Generic function that counts messages
    fn count_messages<S: AeronSubscriber>(subscriber: &mut S) -> Result<usize, TransportError> {
        let mut count = 0;
        subscriber.poll(|_fragment| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }

    // Generic function using zero-copy publication
    fn send_with_claim<P: AeronPublisher>(publisher: &mut P, data: &[u8]) -> Result<(), TransportError> {
        let mut claim = publisher.try_claim(data.len())?;
        claim[0..data.len()].copy_from_slice(data);
        Ok(())
    }

    #[test]
    fn test_generic_heartbeat() {
        let mut publisher = MockPublisher::new();
        let position = send_heartbeat(&mut publisher).unwrap();
        assert_eq!(position, 0);

        let messages = publisher.published_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"HEARTBEAT");
    }

    #[test]
    fn test_generic_count_messages() {
        let mut subscriber = MockSubscriber::new();
        subscriber.inject_message(b"msg1".to_vec());
        subscriber.inject_message(b"msg2".to_vec());
        subscriber.inject_message(b"msg3".to_vec());

        let count = count_messages(&mut subscriber).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_generic_zero_copy() {
        let mut publisher = MockPublisher::new();
        send_with_claim(&mut publisher, b"zero-copy test").unwrap();

        let messages = publisher.published_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(&messages[0][0..14], b"zero-copy test");
    }

    #[test]
    fn test_publisher_subscriber_roundtrip() {
        // This demonstrates a common pattern: publish to one transport,
        // receive from another (in tests, both are mocks)
        let mut publisher = MockPublisher::new();
        let mut subscriber = MockSubscriber::new();

        // Publish some messages
        publisher.offer(b"message 1").unwrap();
        publisher.offer(b"message 2").unwrap();
        publisher.offer(b"message 3").unwrap();

        // Transfer to subscriber (in real system, Aeron handles this)
        for msg in publisher.published_messages() {
            subscriber.inject_message(msg.clone());
        }

        // Verify we can receive them
        let mut received = Vec::new();
        subscriber
            .poll(|fragment| {
                received.push(fragment.as_ref().to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(received.len(), 3);
        assert_eq!(received[0], b"message 1");
        assert_eq!(received[1], b"message 2");
        assert_eq!(received[2], b"message 3");
    }

    #[test]
    fn test_error_propagation() {
        let mut subscriber = MockSubscriber::new();
        subscriber.inject_message(b"msg1".to_vec());
        subscriber.inject_message(b"msg2".to_vec());

        // Handler that returns error on second message
        let mut count = 0;
        let result = subscriber.poll(|_fragment| {
            count += 1;
            if count == 2 {
                Err(TransportError::IoError("simulated error".to_string()))
            } else {
                Ok(())
            }
        });

        assert!(result.is_err());
        match result {
            Err(TransportError::IoError(msg)) => assert_eq!(msg, "simulated error"),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_back_pressure_handling() {
        let mut publisher = MockPublisher::new();
        publisher.simulate_back_pressure_once();

        // First offer should fail with back-pressure
        let result = publisher.offer(b"test");
        assert!(matches!(result, Err(TransportError::BackPressure)));

        // Second offer should succeed
        let result = publisher.offer(b"test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_buffer_metadata_access() {
        let mut subscriber = MockSubscriber::new();
        subscriber.set_session_id(42);
        subscriber.set_stream_id(100);
        subscriber.inject_message(b"test message".to_vec());

        subscriber
            .poll(|fragment| {
                // Access fragment metadata
                assert_eq!(fragment.len(), 12);
                assert_eq!(fragment.position(), 0);
                assert_eq!(fragment.header().session_id, 42);
                assert_eq!(fragment.header().stream_id, 100);

                // Access data
                assert_eq!(fragment.as_ref(), b"test message");
                Ok(())
            })
            .unwrap();
    }
}
