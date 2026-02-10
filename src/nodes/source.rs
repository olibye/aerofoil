//! Mutable source node for externally-driven updates.
//!
//! This module provides [`MutableSource`], a Wingfoil node that can be updated
//! from outside the graph execution model. This is useful for inverted control
//! flow patterns where external event loops (like Aeron polling with idle strategies)
//! drive state updates.

use wingfoil::{Element, GraphState, MutableNode, StreamPeekRef, UpStreams};

/// A Wingfoil node that holds mutable state updated from outside the graph.
///
/// Unlike [`AeronSubscriberValueNode`](crate::nodes::AeronSubscriberValueNode) which
/// polls for messages during its cycle, `MutableSource` is a passive container that
/// can be updated externally. This enables inverted control flow patterns where:
///
/// 1. An external event loop (e.g., Aeron polling with idle strategy) drives execution
/// 2. Zero-copy processing happens in the event callback
/// 3. Extracted values are pushed into `MutableSource` nodes
/// 4. The Wingfoil graph processes downstream from these sources
///
/// # Type Parameters
///
/// - `T`: The value type (must implement `Element` trait)
///
/// # Element Trait
///
/// The value type `T` must implement Wingfoil's `Element` trait, which requires:
/// `Debug + Clone + Default + 'static`. This ensures:
/// - `Debug`: For logging and debugging
/// - `Clone`: For value copying (use `Rc<T>` for large types to make cloning cheap)
/// - `Default`: For providing an initial value
/// - `'static`: No non-static references
///
/// # Use Cases
///
/// ## Zero-Copy SBE Decoding
///
/// Process SBE-encoded messages with zero-copy decoders in the Aeron poll callback,
/// then push extracted fields into source nodes:
///
/// ```rust,no_run
/// use aerofoil::nodes::MutableSource;
/// use aerofoil::transport::{AeronSubscriber, rusteron::RusteronSubscriber};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// # fn example(mut subscriber: RusteronSubscriber) {
/// // Create source nodes for extracted fields
/// let price_source = Rc::new(RefCell::new(MutableSource::new(0.0_f64)));
/// let qty_source = Rc::new(RefCell::new(MutableSource::new(0_i64)));
///
/// // Aeron polling loop with zero-copy SBE decoding
/// subscriber.poll(|fragment| {
///     // Hypothetical SBE decoder (zero-copy, only lives in this callback)
///     // let order = OrderDecoder::wrap(fragment.as_ref(), 0);
///     // let price = order.price();
///     // let qty = order.quantity();
///
///     // Extract values and update sources
///     // price_source.borrow_mut().set(price);
///     // qty_source.borrow_mut().set(qty);
///
///     Ok(())
/// });
///
/// // Wingfoil graph processes from these sources
/// // graph.cycle()?;
/// # }
/// ```
///
/// ## Aeron Idle Strategy Integration
///
/// Integrate Aeron's idle strategies (BusySpinIdleStrategy, BackoffIdleStrategy, etc.)
/// with Wingfoil processing:
///
/// ```rust,no_run
/// # use aerofoil::nodes::MutableSource;
/// # use aerofoil::transport::{AeronSubscriber, rusteron::RusteronSubscriber};
/// # use std::rc::Rc;
/// # use std::cell::RefCell;
/// # fn example(mut subscriber: RusteronSubscriber) -> anyhow::Result<()> {
/// // let idle_strategy = rusteron_client::BusySpinIdleStrategy::new();
/// let source = Rc::new(RefCell::new(MutableSource::new(0_i64)));
///
/// loop {
///     // 1. Poll Aeron (returns work count)
///     let work_count = subscriber.poll(|fragment| {
///         // source.borrow_mut().set(parse(fragment));
///         Ok(())
///     })?;
///
///     // 2. Run Wingfoil graph cycle
///     // graph.cycle()?;
///
///     // 3. Idle strategy manages CPU when no work
///     // idle_strategy.idle(work_count);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # StreamPeekRef Implementation
///
/// `MutableSource` implements `StreamPeekRef<T>`, allowing downstream nodes to
/// access the current value using Wingfoil's standard composition pattern.
///
/// # Example
///
/// See `examples/inverted_control_idle_strategy.rs` for a complete runnable example.
pub struct MutableSource<T: Element> {
    value: T,
}

impl<T: Element> MutableSource<T> {
    /// Creates a new `MutableSource` with the given initial value.
    ///
    /// # Parameters
    ///
    /// - `initial_value`: The initial value before any external updates
    ///
    /// # Returns
    ///
    /// A new `MutableSource` instance ready to be added to a Wingfoil graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aerofoil::nodes::MutableSource;
    ///
    /// let source = MutableSource::new(42_i64);
    /// assert_eq!(*source.get(), 42);
    /// ```
    pub fn new(initial_value: T) -> Self {
        Self {
            value: initial_value,
        }
    }

    /// Updates the value held by this source.
    ///
    /// This method is typically called from outside the graph execution,
    /// such as in an Aeron polling callback or other event handler.
    ///
    /// # Parameters
    ///
    /// - `value`: The new value to store
    ///
    /// # Example
    ///
    /// ```rust
    /// use aerofoil::nodes::MutableSource;
    ///
    /// let mut source = MutableSource::new(0_i64);
    /// source.set(100);
    /// assert_eq!(*source.get(), 100);
    /// ```
    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    /// Returns a reference to the current value.
    ///
    /// # Returns
    ///
    /// A reference to the value currently held by this source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aerofoil::nodes::MutableSource;
    ///
    /// let source = MutableSource::new(42_i64);
    /// assert_eq!(*source.get(), 42);
    /// ```
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Element> StreamPeekRef<T> for MutableSource<T> {
    /// Returns a reference to the current value.
    ///
    /// This implementation enables downstream Wingfoil nodes to access the
    /// source value using the standard `peek_ref()` pattern.
    fn peek_ref(&self) -> &T {
        &self.value
    }
}

impl<T: Element> MutableNode for MutableSource<T> {
    /// Called by Wingfoil on each graph cycle.
    ///
    /// `MutableSource` is passive and performs no work during cycles - it only
    /// holds values updated externally. Returns `false` to indicate the node
    /// should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // No-op: source is updated externally
        Ok(false)
    }

    /// Called when the graph starts.
    ///
    /// `MutableSource` has no upstreams and requires no initialization.
    fn start(&mut self, _state: &mut GraphState) -> anyhow::Result<()> {
        // No initialization required
        Ok(())
    }

    fn upstreams(&self) -> UpStreams {
        UpStreams::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wingfoil::StreamPeek;

    #[test]
    fn given_initial_value_when_created_then_returns_initial_value() {
        let source = MutableSource::new(42_i64);
        assert_eq!(*source.get(), 42);
    }

    #[test]
    fn given_set_when_called_then_updates_value() {
        let mut source = MutableSource::new(0_i64);
        source.set(100);
        assert_eq!(*source.get(), 100);
    }

    #[test]
    fn given_multiple_updates_when_set_then_reflects_latest_value() {
        let mut source = MutableSource::new(0_i64);
        source.set(10);
        source.set(20);
        source.set(30);
        assert_eq!(*source.get(), 30);
    }

    #[test]
    fn given_peek_ref_when_called_then_returns_reference_to_value() {
        let source = MutableSource::new(99_i64);
        let value_ref = source.peek_ref();
        assert_eq!(*value_ref, 99);
    }

    #[test]
    fn given_rc_refcell_when_used_then_supports_peek_value() {
        // Wingfoil auto-implements StreamPeek for RefCell<T> where T: StreamPeekRef
        let source = Rc::new(RefCell::new(MutableSource::new(42_i64)));

        // peek_value() is available on Rc<RefCell<...>>
        let value = source.peek_value();
        assert_eq!(value, 42);

        // Can update through RefCell
        source.borrow_mut().set(100);
        assert_eq!(source.peek_value(), 100);
    }

    #[test]
    fn given_custom_element_type_when_used_then_works() {
        #[derive(Debug, Clone, Default, PartialEq)]
        struct Trade {
            price: f64,
            quantity: i64,
        }
        // Element is automatically implemented for types that are Debug + Clone + Default + 'static

        let mut source = MutableSource::new(Trade::default());

        source.set(Trade {
            price: 123.45,
            quantity: 100,
        });

        assert_eq!(source.get().price, 123.45);
        assert_eq!(source.get().quantity, 100);
    }

    #[test]
    fn given_upstreams_when_called_then_returns_none() {
        let source = MutableSource::new(0_i64);
        let upstreams = source.upstreams();

        // UpStreams::none() should indicate no upstreams
        assert_eq!(upstreams.active.len(), 0);
        assert_eq!(upstreams.passive.len(), 0);
    }
}
