# Mocking Strategy

This document explains when and how to use mockall for test mocking in the aerofoil project.

## When to Use Mockall

Use mockall's `#[automock]` attribute for traits that have:

- **Simple method signatures**: Methods without complex lifetime parameters
- **No generic closures**: Trait methods that don't take generic `FnMut` or `Fn` parameters
- **Standard return types**: Methods returning simple types, `Result<T, E>`, or boxed futures
- **No associated lifetimes in generics**: Avoid `#[automock]` if method generics reference `self`'s lifetime

### Example - Good for mockall

```rust
#[cfg_attr(test, automock)]
pub trait DataStore {
    fn get(&self, key: &str) -> Result<String, Error>;
    fn put(&mut self, key: String, value: String) -> Result<(), Error>;
}
```

**In tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_retrieval() {
        // Given
        let mut mock = MockDataStore::new();
        mock.expect_get()
            .with(eq("key1"))
            .returning(|_| Ok("value1".to_string()));

        // When
        let result = mock.get("key1");

        // Then
        assert_eq!(result.unwrap(), "value1");
    }
}
```

## When NOT to Use Mockall

Provide **manual test implementations** for traits that have:

- **Explicit lifetime parameters**: Methods like `fn claim<'a>(&'a mut self) -> Buffer<'a>`
- **Generic closure parameters**: Methods like `fn poll<F: FnMut(&Item)>(&mut self, handler: F)`
- **Complex associated types with lifetimes**: GATs (Generic Associated Types) with lifetime constraints
- **High-rank trait bounds**: Methods with `for<'a>` syntax

### Why Mockall Fails on These

- Mockall's procedural macro cannot generate correct lifetime relationships
- Generic closures create lifetime constraints that mockall can't express
- Results in compilation errors like "parameter type may not live long enough"

### Example - Manual Implementation Required

```rust
// DON'T use #[automock] - mockall can't handle this
pub trait AeronPublisher {
    fn try_claim<'a>(&'a mut self, len: usize) -> Result<Buffer<'a>, Error>;
}

// Instead, implement manually for tests:
#[cfg(test)]
struct TestPublisher {
    buffers: Vec<Vec<u8>>,
}

#[cfg(test)]
impl AeronPublisher for TestPublisher {
    fn try_claim<'a>(&'a mut self, len: usize) -> Result<Buffer<'a>, Error> {
        self.buffers.push(vec![0u8; len]);
        Ok(Buffer::new(self.buffers.last_mut().unwrap()))
    }
}
```

**In tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_claim() {
        // Given
        let mut publisher = TestPublisher { buffers: Vec::new() };

        // When
        let mut claim = publisher.try_claim(256).unwrap();
        claim[0..5].copy_from_slice(b"hello");

        // Then
        assert_eq!(&publisher.buffers[0][0..5], b"hello");
    }
}
```

## Guidelines for Manual Test Implementations

When mockall is not suitable:

1. **Keep it simple**: Manual test implementations should be ~10-30 lines
2. **Store test data**: Use `Vec`, `VecDeque`, or similar collections to capture calls
3. **Add helper methods**: Provide inspection methods (e.g., `messages()`) for assertions
4. **Document the pattern**: Add comments explaining why manual implementation is needed
5. **Provide examples**: Include manual implementations in `tests.rs` as examples for users

### Pattern: Capturing Method Calls

```rust
#[cfg(test)]
struct TestPublisher {
    messages: Vec<Vec<u8>>,
    next_position: i64,
}

#[cfg(test)]
impl TestPublisher {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_position: 0,
        }
    }

    // Helper method for test assertions
    fn messages(&self) -> &[Vec<u8>] {
        &self.messages
    }
}

#[cfg(test)]
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
```

### Pattern: Injecting Test Data

```rust
#[cfg(test)]
struct TestSubscriber {
    messages: VecDeque<Vec<u8>>,
    next_position: i64,
}

#[cfg(test)]
impl TestSubscriber {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            next_position: 0,
        }
    }

    // Helper method to inject test messages
    fn inject(&mut self, data: Vec<u8>) {
        self.messages.push_back(data);
    }
}

#[cfg(test)]
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
```

## Decision Tree

Use this flowchart to decide which mocking approach to use:

```
Does the trait have any of these?
  - Methods with explicit lifetime parameters like <'a>
  - Methods with generic closures like <F: FnMut>
  - GATs or complex associated types

  YES → Use manual test implementation
   |
   └─→ See "Guidelines for Manual Test Implementations"

  NO → Use mockall's #[automock]
   |
   └─→ See "When to Use Mockall"
```

## References

- Real-world examples: See `src/transport/tests.rs` for manual implementations
- Mockall documentation: https://github.com/asomers/mockall
- Mockall limitations: https://github.com/asomers/mockall/blob/master/mockall/examples/limitations.md
