//! Demonstrates inverted control flow with MutableSource and Aeron idle strategy.
//!
//! This example shows:
//! - Using MutableSource<T> for externally-driven updates
//! - Aeron polling loop with idle strategy
//! - Zero-copy message processing in poll callbacks
//! - Simulated SBE-style decoding (decoder only lives in callback)
//! - Wingfoil graph processing from source nodes
//!
//! # Architecture Pattern
//!
//! Traditional (Wingfoil-driven):
//! ```text
//! Graph Cycle → AeronSubscriberNode.poll() → Parse → Store owned T → Peek
//! ```
//!
//! Inverted Control (Aeron-driven):
//! ```text
//! Aeron poll() → Zero-copy decode → Extract fields → Update Sources
//!     ↓
//! Graph Cycle → Read from Sources → Process
//!     ↓
//! Idle Strategy (manage CPU)
//! ```
//!
//! # Benefits
//!
//! - Zero-copy decoding: Decoders only live during poll callback
//! - Proper idle strategy: Aeron controls CPU management
//! - Efficient: Only cycle graph when messages arrive
//! - Single-threaded: No context switches for HFT
//!
//! Run with: `cargo run --example inverted_control_idle_strategy`
//!
//! Note: Requires the Aeron media driver to be running.
//! See openspec/integration-test.md for setup instructions.

use aerofoil::nodes::MutableSource;
use aerofoil::transport::rusteron::RusteronSubscriber;
use aerofoil::transport::AeronSubscriber;
use rusteron_client::IntoCString;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use wingfoil::{
    Graph, GraphState, IntoNode, MutableNode, RunFor, RunMode, StreamPeek, StreamPeekRef, UpStreams,
};

/// Simulated SBE decoder that only borrows from the buffer.
///
/// In real SBE usage, this would be generated code that wraps a buffer
/// with zero-copy accessors. The decoder CANNOT be stored - it only lives
/// during the fragment callback.
struct OrderDecoder<'a> {
    buffer: &'a [u8],
}

impl<'a> OrderDecoder<'a> {
    /// Wrap a buffer for zero-copy decoding.
    ///
    /// Real SBE decoders follow this pattern - they take a borrowed buffer
    /// and provide accessor methods.
    fn wrap(buffer: &'a [u8], _offset: usize) -> Option<Self> {
        if buffer.len() >= 24 {
            Some(Self { buffer })
        } else {
            None
        }
    }

    /// Decode price field (zero-copy accessor).
    fn price(&self) -> f64 {
        f64::from_le_bytes(self.buffer[0..8].try_into().unwrap())
    }

    /// Decode quantity field (zero-copy accessor).
    fn quantity(&self) -> i64 {
        i64::from_le_bytes(self.buffer[8..16].try_into().unwrap())
    }

    /// Decode side field (zero-copy accessor).
    fn side(&self) -> char {
        self.buffer[16] as char
    }
}

/// Downstream processing node that consumes from source nodes.
///
/// This node demonstrates the standard Wingfoil composition pattern,
/// accessing upstream values via peek_value().
struct OrderProcessorNode<P, Q, S>
where
    P: StreamPeekRef<f64>,
    Q: StreamPeekRef<i64>,
    S: StreamPeekRef<char>,
{
    price_source: Rc<RefCell<P>>,
    qty_source: Rc<RefCell<Q>>,
    side_source: Rc<RefCell<S>>,
    total_volume: i64,
    message_count: usize,
}

impl<P, Q, S> OrderProcessorNode<P, Q, S>
where
    P: StreamPeekRef<f64>,
    Q: StreamPeekRef<i64>,
    S: StreamPeekRef<char>,
{
    fn new(
        price_source: Rc<RefCell<P>>,
        qty_source: Rc<RefCell<Q>>,
        side_source: Rc<RefCell<S>>,
    ) -> Self {
        Self {
            price_source,
            qty_source,
            side_source,
            total_volume: 0,
            message_count: 0,
        }
    }

    fn print_stats(&self) {
        println!("  Total volume processed: {}", self.total_volume);
        println!("  Messages processed: {}", self.message_count);
    }
}

impl<P, Q, S> MutableNode for OrderProcessorNode<P, Q, S>
where
    P: StreamPeekRef<f64> + 'static,
    Q: StreamPeekRef<i64> + 'static,
    S: StreamPeekRef<char> + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // Access values from source nodes using peek_value()
        let price = self.price_source.peek_value();
        let qty = self.qty_source.peek_value();
        let side = self.side_source.peek_value();

        // Skip default/initial values (indicates no message processed yet)
        if price == 0.0 {
            return Ok(false);
        }

        // Process the order
        self.message_count += 1;
        self.total_volume += qty;

        println!(
            "  Processed: {} {} @ ${:.2} (total volume: {})",
            side, qty, price, self.total_volume
        );

        Ok(false)
    }

    fn start(&mut self, _state: &mut GraphState) -> anyhow::Result<()> {
        Ok(())
    }

    fn upstreams(&self) -> UpStreams {
        UpStreams::none()
    }
}

fn main() {
    println!("Inverted Control + Idle Strategy Pattern");
    println!("=========================================");
    println!();
    println!("This example demonstrates:");
    println!("- MutableSource<T> for externally-driven updates");
    println!("- Aeron-driven polling with idle strategy");
    println!("- Zero-copy decoding (simulated SBE pattern)");
    println!("- Wingfoil graph processing from sources");
    println!();

    // Create Aeron context and connection
    let context = match rusteron_client::AeronContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to create Aeron context: {:?}", e);
            eprintln!("Make sure the Aeron media driver is running.");
            eprintln!("See openspec/integration-test.md for setup instructions.");
            return;
        }
    };

    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let stream_id = 3003;

    // Create subscriber
    let async_sub = aeron
        .async_add_subscription(
            &channel.into_c_string(),
            stream_id,
            rusteron_client::Handlers::no_available_image_handler(),
            rusteron_client::Handlers::no_unavailable_image_handler(),
        )
        .expect("Failed to start subscription");

    let subscription = async_sub
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete subscription");

    let mut subscriber = RusteronSubscriber::new(subscription);

    // Create source nodes for extracted fields
    println!("Creating source nodes for price, quantity, and side...");

    // Create Rc<RefCell<>> wrapped sources for sharing between poll callback and graph
    let price_source = Rc::new(RefCell::new(MutableSource::new(0.0_f64)));
    let qty_source = Rc::new(RefCell::new(MutableSource::new(0_i64)));
    let side_source = Rc::new(RefCell::new(MutableSource::new(' ')));

    // Clone references for poll callback
    let price_source_for_poll = price_source.clone();
    let qty_source_for_poll = qty_source.clone();
    let side_source_for_poll = side_source.clone();

    // Create downstream processing node (uses clones)
    let processor = OrderProcessorNode::new(
        price_source.clone(),
        qty_source.clone(),
        side_source.clone(),
    );

    // Build Wingfoil graph with sources and processor
    // Note: RunFor::Cycles(1) because we'll call run() in our own loop
    // Rc<RefCell<MutableSource>> can be passed directly (coerces to Rc<dyn Node>)
    let mut graph = Graph::new(
        vec![
            price_source,
            qty_source,
            side_source,
            RefCell::new(processor).into_node(),
        ],
        RunMode::RealTime,
        RunFor::Cycles(1), // Run one cycle per iteration
    );

    println!("Graph built with 3 sources + 1 processor node");
    println!();

    // Simulate idle strategy (rusteron doesn't expose idle strategies directly)
    // In production HFT, you'd use: rusteron_client::BusySpinIdleStrategy::new()
    let max_cycles = 100;
    let mut cycle_count = 0;
    let start_time = Instant::now();

    println!("Starting Aeron-driven polling loop...");
    println!("(Publish 24-byte messages to aeron:ipc stream {} to see processing)", stream_id);
    println!("Message format: [f64 price][i64 qty][char side][padding]");
    println!();

    loop {
        // 1. Poll Aeron (this is where zero-copy SBE decoding happens)
        let work_count = match subscriber.poll(|fragment| {
            // Zero-copy decoder lives ONLY in this callback
            if let Some(decoder) = OrderDecoder::wrap(fragment.as_ref(), 0) {
                // Extract fields immediately (decoder borrows from fragment)
                let price = decoder.price();
                let qty = decoder.quantity();
                let side = decoder.side();

                // Update source nodes with extracted values
                price_source_for_poll.borrow_mut().set(price);
                qty_source_for_poll.borrow_mut().set(qty);
                side_source_for_poll.borrow_mut().set(side);

                println!("Aeron poll: decoded order {} {} @ ${:.2}", side, qty, price);
            }
            // Decoder is dropped here - cannot be stored!
            Ok(())
        }) {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Poll error: {:?}", e);
                0
            }
        };

        // 2. Run Wingfoil graph cycle (processes from updated sources)
        if let Err(e) = graph.run() {
            eprintln!("Graph run error: {:?}", e);
            break;
        }

        // 3. Idle strategy would go here
        // In real HFT code:
        // idle_strategy.idle(work_count);
        //
        // For this example, we just sleep briefly to avoid busy spinning
        if work_count == 0 {
            std::thread::sleep(Duration::from_millis(10));
        }

        cycle_count += 1;
        if cycle_count >= max_cycles {
            break;
        }
    }

    let elapsed = start_time.elapsed();
    println!();
    println!("Completed {} cycles in {:?}", cycle_count, elapsed);
    println!(
        "Average cycle time: {:?}",
        elapsed / cycle_count as u32
    );

    // Print final stats from processor node
    println!();
    println!("Final Statistics:");
    // We can't easily access the processor here without more plumbing,
    // but in real code you'd keep a reference to print final stats

    println!();
    println!("Example completed successfully!");
    println!();
    println!("Key takeaways:");
    println!("- Decoders never left the poll callback (zero-copy)");
    println!("- Sources were updated externally, not polled internally");
    println!("- Graph cycles ran independently of polling");
    println!("- Idle strategy would manage CPU when no messages");
}
