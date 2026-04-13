//! Aeron publisher node and extension trait for Wingfoil stream composition.

use crate::nodes::MutableSource;
use crate::transport::{AeronPublisher, AeronStatus, TransportError};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use wingfoil::{Element, GraphState, MutableNode, StreamPeekRef, UpStreams};

/// A Wingfoil node that reads upstream values and publishes them via Aeron.
///
/// Each `cycle()`:
/// 1. Peeks the upstream `StreamPeekRef<T>` for the current value
/// 2. If changed from the last published value, serializes and calls `offer()`
/// 3. On `BackPressure`, surfaces the state via the optional status stream and retries next cycle
pub struct AeronPublisherNode<T, P, Ser, U>
where
    T: Element,
    P: AeronPublisher,
    Ser: FnMut(&T) -> Vec<u8>,
    U: StreamPeekRef<T>,
{
    upstream: Rc<RefCell<U>>,
    publisher: P,
    serializer: Ser,
    last_value: T,
    status: Option<Rc<RefCell<MutableSource<AeronStatus>>>>,
    last_status: AeronStatus,
    _marker: PhantomData<T>,
}

impl<T, P, Ser, U> std::fmt::Debug for AeronPublisherNode<T, P, Ser, U>
where
    T: Element,
    P: AeronPublisher,
    Ser: FnMut(&T) -> Vec<u8>,
    U: StreamPeekRef<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AeronPublisherNode").finish_non_exhaustive()
    }
}

impl<T, P, Ser, U> AeronPublisherNode<T, P, Ser, U>
where
    T: Element + PartialEq,
    P: AeronPublisher,
    Ser: FnMut(&T) -> Vec<u8>,
    U: StreamPeekRef<T>,
{
    /// Creates a new publisher node without a status stream.
    pub fn new(upstream: Rc<RefCell<U>>, publisher: P, serializer: Ser) -> Self {
        Self {
            upstream,
            publisher,
            serializer,
            last_value: T::default(),
            status: None,
            last_status: AeronStatus::Disconnected,
            _marker: PhantomData,
        }
    }

    /// Creates a new publisher node that reports lifecycle transitions on the given status stream.
    pub fn with_status(
        upstream: Rc<RefCell<U>>,
        publisher: P,
        serializer: Ser,
        status: Rc<RefCell<MutableSource<AeronStatus>>>,
    ) -> Self {
        Self {
            upstream,
            publisher,
            serializer,
            last_value: T::default(),
            status: Some(status),
            last_status: AeronStatus::Disconnected,
            _marker: PhantomData,
        }
    }

    fn update_status(&mut self, new_status: AeronStatus) {
        if let Some(status) = &self.status {
            if new_status != self.last_status {
                status.borrow_mut().set(new_status.clone());
                self.last_status = new_status;
            }
        }
    }

    fn poll_and_publish(&mut self) -> anyhow::Result<()> {
        // Closed is terminal — surface it before attempting to publish
        if self.publisher.is_closed() {
            self.update_status(AeronStatus::Closed);
            return Ok(());
        }

        let upstream = self.upstream.borrow();
        let current = upstream.peek_ref();
        let mut offer_outcome: Option<AeronStatus> = None;
        if *current != self.last_value {
            let bytes = (self.serializer)(current);
            let cloned = current.clone();
            drop(upstream);
            match self.publisher.offer(&bytes) {
                Ok(_) => {
                    self.last_value = cloned;
                    offer_outcome = Some(AeronStatus::Connected);
                }
                Err(TransportError::BackPressure) => {
                    offer_outcome = Some(AeronStatus::BackPressured);
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            drop(upstream);
        }

        // If we didn't attempt an offer this cycle, fall back to the publisher's
        // connection state so consumers see Disconnected/Connected transitions
        // even when upstream hasn't ticked.
        let new_status = offer_outcome.unwrap_or_else(|| {
            if self.publisher.is_connected() {
                AeronStatus::Connected
            } else {
                AeronStatus::Disconnected
            }
        });
        self.update_status(new_status);

        Ok(())
    }
}

impl<T, P, Ser, U> MutableNode for AeronPublisherNode<T, P, Ser, U>
where
    T: Element + PartialEq,
    P: AeronPublisher + 'static,
    Ser: FnMut(&T) -> Vec<u8> + 'static,
    U: StreamPeekRef<T> + MutableNode + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        self.poll_and_publish()?;
        Ok(false)
    }

    fn start(&mut self, state: &mut GraphState) -> anyhow::Result<()> {
        state.always_callback();
        Ok(())
    }

    fn upstreams(&self) -> UpStreams {
        let node: Rc<dyn wingfoil::Node> = self.upstream.clone();
        UpStreams::new(vec![node], vec![])
    }
}

/// Return type for [`AeronPub::aeron_pub_with_status`]: `(publisher_node, status_source)`.
pub type DualStreamPublisher<T, P, Ser, U> = (
    Rc<RefCell<AeronPublisherNode<T, P, Ser, U>>>,
    Rc<RefCell<MutableSource<AeronStatus>>>,
);

/// Extension trait for creating Aeron publisher nodes from upstream streams.
///
/// Implemented on `Rc<RefCell<U>>` where `U: StreamPeekRef<T>`, mirroring
/// Wingfoil's `ZeroMqPub<T>` extension pattern.
pub trait AeronPub<T: Element, U: StreamPeekRef<T>> {
    /// Creates an `AeronPublisherNode` that publishes upstream values via Aeron.
    fn aeron_pub<P, Ser>(&self, publisher: P, serializer: Ser) -> AeronPublisherNode<T, P, Ser, U>
    where
        P: AeronPublisher,
        Ser: FnMut(&T) -> Vec<u8>;

    /// Creates a publisher node and a paired status source. The status source emits
    /// `AeronStatus` transitions (Connected, Disconnected, BackPressured, Closed)
    /// derived from `offer()` results and `is_connected()` / `is_closed()` polling.
    fn aeron_pub_with_status<P, Ser>(
        &self,
        publisher: P,
        serializer: Ser,
    ) -> DualStreamPublisher<T, P, Ser, U>
    where
        P: AeronPublisher,
        Ser: FnMut(&T) -> Vec<u8>;
}

impl<T, U> AeronPub<T, U> for Rc<RefCell<U>>
where
    T: Element + PartialEq,
    U: StreamPeekRef<T> + MutableNode + 'static,
{
    fn aeron_pub<P, Ser>(&self, publisher: P, serializer: Ser) -> AeronPublisherNode<T, P, Ser, U>
    where
        P: AeronPublisher,
        Ser: FnMut(&T) -> Vec<u8>,
    {
        AeronPublisherNode::new(Rc::clone(self), publisher, serializer)
    }

    fn aeron_pub_with_status<P, Ser>(
        &self,
        publisher: P,
        serializer: Ser,
    ) -> DualStreamPublisher<T, P, Ser, U>
    where
        P: AeronPublisher,
        Ser: FnMut(&T) -> Vec<u8>,
    {
        let status = Rc::new(RefCell::new(MutableSource::new(AeronStatus::Disconnected)));
        let node = AeronPublisherNode::with_status(
            Rc::clone(self),
            publisher,
            serializer,
            Rc::clone(&status),
        );
        (Rc::new(RefCell::new(node)), status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ClaimBuffer, TransportError};

    struct MockPublisher {
        offered: RefCell<Vec<Vec<u8>>>,
        back_pressure: bool,
        connected: bool,
        closed: bool,
    }

    impl MockPublisher {
        fn new() -> Self {
            Self {
                offered: RefCell::new(Vec::new()),
                back_pressure: false,
                connected: true,
                closed: false,
            }
        }

        fn with_back_pressure() -> Self {
            Self {
                offered: RefCell::new(Vec::new()),
                back_pressure: true,
                connected: true,
                closed: false,
            }
        }

        fn closed() -> Self {
            Self {
                offered: RefCell::new(Vec::new()),
                back_pressure: false,
                connected: false,
                closed: true,
            }
        }
    }

    impl AeronPublisher for MockPublisher {
        fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
            if self.back_pressure {
                return Err(TransportError::BackPressure);
            }
            self.offered.borrow_mut().push(buffer.to_vec());
            Ok(0)
        }

        fn try_claim(&mut self, _length: usize) -> Result<ClaimBuffer<'_>, TransportError> {
            Err(TransportError::Invalid("mock".to_string()))
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn is_closed(&self) -> bool {
            self.closed
        }
    }

    struct MockSource {
        value: i64,
    }

    impl MutableNode for MockSource {
        fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
            Ok(false)
        }

        fn start(&mut self, _state: &mut GraphState) -> anyhow::Result<()> {
            Ok(())
        }

        fn upstreams(&self) -> UpStreams {
            UpStreams::none()
        }
    }

    impl StreamPeekRef<i64> for MockSource {
        fn peek_ref(&self) -> &i64 {
            &self.value
        }
    }

    #[test]
    fn given_publisher_node_when_cycle_then_offers_serialized_upstream() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let mut node = AeronPublisherNode::new(Rc::clone(&source), publisher, serializer);
        node.poll_and_publish().unwrap();

        let offered = node.publisher.offered.borrow();
        assert_eq!(offered.len(), 1);
        assert_eq!(offered[0], 42i64.to_le_bytes().to_vec());
    }

    #[test]
    fn given_back_pressure_when_offer_then_skips_without_error() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::with_back_pressure();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let mut node = AeronPublisherNode::new(Rc::clone(&source), publisher, serializer);
        let result = node.poll_and_publish();
        assert!(result.is_ok());
    }

    #[test]
    fn given_unchanged_value_when_cycle_then_does_not_re_offer() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let mut node = AeronPublisherNode::new(Rc::clone(&source), publisher, serializer);
        node.poll_and_publish().unwrap();
        node.poll_and_publish().unwrap();

        let offered = node.publisher.offered.borrow();
        assert_eq!(offered.len(), 1);
    }

    #[test]
    fn given_changed_value_when_cycle_then_offers_new_value() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let mut node = AeronPublisherNode::new(Rc::clone(&source), publisher, serializer);
        node.poll_and_publish().unwrap();
        source.borrow_mut().value = 100;
        node.poll_and_publish().unwrap();

        let offered = node.publisher.offered.borrow();
        assert_eq!(offered.len(), 2);
        assert_eq!(offered[1], 100i64.to_le_bytes().to_vec());
    }

    #[test]
    fn given_extension_trait_when_aeron_pub_then_creates_working_node() {
        let source = Rc::new(RefCell::new(MockSource { value: 77 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let mut node = source.aeron_pub(publisher, serializer);
        node.poll_and_publish().unwrap();

        let offered = node.publisher.offered.borrow();
        assert_eq!(offered.len(), 1);
        assert_eq!(offered[0], 77i64.to_le_bytes().to_vec());
    }

    // --- Dual-stream publisher status tests ---

    #[test]
    fn given_successful_offer_when_publish_then_status_transitions_to_connected() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let (node, status) = source.aeron_pub_with_status(publisher, serializer);
        node.borrow_mut().poll_and_publish().unwrap();

        assert_eq!(*status.borrow().get(), AeronStatus::Connected);
    }

    #[test]
    fn given_back_pressure_when_publish_then_status_transitions_to_back_pressured() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::with_back_pressure();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let (node, status) = source.aeron_pub_with_status(publisher, serializer);
        node.borrow_mut().poll_and_publish().unwrap();

        assert_eq!(*status.borrow().get(), AeronStatus::BackPressured);
    }

    #[test]
    fn given_closed_publisher_when_publish_then_status_transitions_to_closed() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::closed();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let (node, status) = source.aeron_pub_with_status(publisher, serializer);
        node.borrow_mut().poll_and_publish().unwrap();

        assert_eq!(*status.borrow().get(), AeronStatus::Closed);
    }

    #[test]
    fn given_status_unchanged_when_cycle_then_no_status_re_emit() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::new();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        let (node, status) = source.aeron_pub_with_status(publisher, serializer);
        node.borrow_mut().poll_and_publish().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Connected);

        // second cycle, same value, same publisher state — should remain Connected
        node.borrow_mut().poll_and_publish().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Connected);
    }

    #[test]
    fn given_no_status_stream_when_publish_then_works_silently() {
        let source = Rc::new(RefCell::new(MockSource { value: 42 }));
        let publisher = MockPublisher::with_back_pressure();
        let serializer = |v: &i64| v.to_le_bytes().to_vec();

        // Plain aeron_pub (no status stream) — should still complete cleanly on back-pressure
        let mut node = source.aeron_pub(publisher, serializer);
        node.poll_and_publish().unwrap();
    }
}
