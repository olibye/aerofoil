//! Publisher for testing the inverted_control_idle_strategy example.
//!
//! This example publishes simulated order messages to test the inverted
//! control flow pattern.
//!
//! Run this in a separate terminal while inverted_control_idle_strategy.rs
//! is running to see the pattern in action.
//!
//! Run with: `cargo run --example inverted_control_publisher`

use aerofoil::transport::rusteron::RusteronPublisher;
use aerofoil::transport::AeronPublisher;
use rusteron_client::IntoCString;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Order Publisher for Inverted Control Example");
    println!("=============================================");
    println!();

    // Create Aeron context and connection
    let context = match rusteron_client::AeronContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to create Aeron context: {:?}", e);
            eprintln!("Make sure the Aeron media driver is running.");
            return;
        }
    };

    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let stream_id = 3003;

    // Create publisher
    let async_pub = aeron
        .async_add_publication(&channel.into_c_string(), stream_id)
        .expect("Failed to start publication");

    let publication = async_pub
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete publication");

    let mut publisher = RusteronPublisher::new(publication);

    println!("Publishing to stream {}...", stream_id);
    println!();

    // Simulate order book messages
    let orders: Vec<(f64, i64, char)> = vec![
        (100.50, 1000, 'B'), // Buy 1000 @ 100.50
        (100.75, 500, 'S'),  // Sell 500 @ 100.75
        (100.25, 2000, 'B'), // Buy 2000 @ 100.25
        (101.00, 1500, 'S'), // Sell 1500 @ 101.00
        (100.50, 750, 'B'),  // Buy 750 @ 100.50
    ];

    let order_count = orders.len();

    for (price, qty, side) in orders {
        // Encode message: [f64 price][i64 qty][char side][7 bytes padding]
        let mut buffer = Vec::with_capacity(24);
        buffer.extend_from_slice(&price.to_le_bytes());
        buffer.extend_from_slice(&qty.to_le_bytes());
        buffer.push(side as u8);
        buffer.resize(24, 0); // Pad to 24 bytes

        // Publish with retry on back-pressure
        loop {
            match publisher.offer(&buffer) {
                Ok(position) => {
                    println!(
                        "Published: {} {} @ ${:.2} (position: {})",
                        side, qty, price, position
                    );
                    break;
                }
                Err(e) => {
                    eprintln!("Back-pressure, retrying: {:?}", e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }

        // Small delay between messages
        thread::sleep(Duration::from_millis(100));
    }

    println!();
    println!("Published {} orders", order_count);
    println!("Keeping connection alive for 5 seconds...");
    thread::sleep(Duration::from_secs(5));
}
