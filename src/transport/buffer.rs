use std::ops::{Deref, DerefMut};

/// A claimed buffer for zero-copy message publication.
///
/// This buffer provides mutable access to a region of the Aeron publication buffer.
/// Messages can be written directly into this buffer, avoiding intermediate copies.
///
/// # Design Decision: Lifetime-Bound Ownership
///
/// The lifetime `'a` ensures the buffer cannot outlive the underlying transport
/// resource, preventing use-after-free errors:
///
/// - **Compile-time safety**: Rust borrow checker prevents buffer escaping
/// - **Zero runtime cost**: No reference counting or runtime checks needed
/// - **Aeron semantics**: Matches Aeron's buffer lifecycle model
/// - **Exclusive access**: `&mut` ensures no concurrent access to the same buffer
///
/// # Design Decision: Deref Implementation
///
/// `ClaimBuffer` implements `Deref<Target = [u8]>` for ergonomic access:
///
/// - **Familiar API**: Works like a slice (indexing, iteration, copy_from_slice)
/// - **No wrapper overhead**: Direct access to underlying buffer
/// - **Type safety**: Still a distinct type, not confused with regular slices
///
/// # Design Decision: Position Tracking
///
/// Each claim stores its stream position for correlation and debugging:
///
/// - **Message ordering**: Can verify sequential publication
/// - **Troubleshooting**: Helps diagnose message loss or duplication
/// - **Monitoring**: Enables position-based metrics
///
/// # Safety
///
/// While this type is safe to use, care must be taken to:
/// - Write valid message data before committing
/// - Not exceed the buffer's length
/// - Commit or abort the claim before dropping
///
/// # Example
///
/// ```ignore
/// let mut claim = publisher.try_claim(256)?;
/// claim[0..5].copy_from_slice(b"hello");
/// claim.commit(5)?;
/// ```
pub struct ClaimBuffer<'a> {
    buffer: &'a mut [u8],
    position: i64,
}

impl<'a> ClaimBuffer<'a> {
    /// Creates a new claim buffer wrapping the given mutable slice.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The underlying buffer to wrap
    /// * `position` - The position in the stream where this buffer starts
    pub fn new(buffer: &'a mut [u8], position: i64) -> Self {
        ClaimBuffer { buffer, position }
    }

    /// Returns the length of the claimable buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the position in the stream for this claim.
    pub fn position(&self) -> i64 {
        self.position
    }

    /// Returns the capacity of the buffer.
    ///
    /// This is the same as `len()` for claim buffers.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }
}

impl<'a> Deref for ClaimBuffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

impl<'a> DerefMut for ClaimBuffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer
    }
}

/// A received message fragment buffer for zero-copy message subscription.
///
/// This buffer provides read-only access to a message fragment received from
/// the Aeron subscription. The data is accessed directly from the Aeron buffer
/// without copying.
///
/// # Design Decision: Read-Only Access
///
/// Unlike `ClaimBuffer`, this uses `&[u8]` (immutable) rather than `&mut [u8]`:
///
/// - **Aeron semantics**: Received data is immutable in Aeron
/// - **Safety**: Prevents accidental modification of received messages
/// - **Share-able**: Multiple readers could access (though not in current API)
/// - **Clear intent**: Receiving is distinct from publishing
///
/// # Design Decision: Fragment Header Separation
///
/// Metadata is stored in a separate `FragmentHeader` struct:
///
/// - **Structured access**: Type-safe access to position, session, stream IDs
/// - **Copyable**: Header can be copied without copying message data
/// - **Extensible**: Can add fields without changing buffer structure
/// - **Aeron alignment**: Matches Aeron's header + payload model
///
/// # Design Decision: AsRef Implementation
///
/// Implements both `Deref` and `AsRef<[u8]>`:
///
/// - **Generic APIs**: Works with functions expecting `AsRef<[u8]>`
/// - **Slice operations**: Can use slice methods directly
/// - **Type conversion**: Explicit conversion when needed
///
/// # Lifetime
///
/// The lifetime `'a` ensures the buffer cannot outlive the underlying transport
/// resource and the poll operation that provided it.
///
/// # Example
///
/// ```ignore
/// subscriber.poll(|fragment| {
///     println!("Received {} bytes", fragment.len());
///     process_message(fragment.as_ref());
///     Ok(())
/// })?;
/// ```
pub struct FragmentBuffer<'a> {
    buffer: &'a [u8],
    header: FragmentHeader,
}

/// Metadata about a received message fragment.
///
/// # Design Decision: Public Fields
///
/// Fields are public rather than using getter methods:
///
/// - **Zero-cost access**: No method call overhead
/// - **Copyable type**: No risk of mutation
/// - **Common pattern**: Standard in Rust for simple data structs
/// - **Ergonomic**: Simpler to use (e.g., `header.position` vs `header.position()`)
#[derive(Debug, Clone, Copy)]
pub struct FragmentHeader {
    /// Position in the stream where this fragment starts
    pub position: i64,

    /// Session ID for this fragment
    pub session_id: i32,

    /// Stream ID for this fragment
    pub stream_id: i32,
}

impl<'a> FragmentBuffer<'a> {
    /// Creates a new fragment buffer wrapping the given slice with header metadata.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The underlying buffer containing the message data
    /// * `header` - Metadata about the fragment
    pub fn new(buffer: &'a [u8], header: FragmentHeader) -> Self {
        FragmentBuffer { buffer, header }
    }

    /// Returns the length of the fragment data.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the fragment is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the fragment header metadata.
    pub fn header(&self) -> &FragmentHeader {
        &self.header
    }

    /// Returns the position in the stream for this fragment.
    pub fn position(&self) -> i64 {
        self.header.position
    }
}

impl<'a> Deref for FragmentBuffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

impl<'a> AsRef<[u8]> for FragmentBuffer<'a> {
    fn as_ref(&self) -> &[u8] {
        self.buffer
    }
}
