//! Zero-copy buffer types for transport operations.

use std::ops::{Deref, DerefMut};

/// A claimed buffer for zero-copy message publication.
///
/// This buffer provides mutable access to a region of the Aeron publication buffer.
/// Messages can be written directly into this buffer, avoiding intermediate copies.
///
/// The lifetime `'a` ensures the buffer cannot outlive the underlying transport
/// resource, preventing use-after-free errors.
#[derive(Debug)]
pub struct ClaimBuffer<'a> {
    buffer: &'a mut [u8],
    position: i64,
}

impl<'a> ClaimBuffer<'a> {
    /// Creates a new claim buffer wrapping the given mutable slice.
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
#[derive(Debug)]
pub struct FragmentBuffer<'a> {
    buffer: &'a [u8],
    header: FragmentHeader,
}

/// Metadata about a received message fragment.
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
