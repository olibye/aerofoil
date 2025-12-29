## Why
Currently, the `AeronSubscriberNodeBuilder` requires exact types for its components. Using `Into` and `TryInto` traits will allow for more flexible and ergonomic node construction, enabling automatic type conversions (e.g., passing a literal that converts to the default value).

## What Changes
- Update `AeronSubscriberNodeBuilder` methods (`subscriber`, `default`) to accept types implementing `Into` where appropriate.
- Update tests to verify the flexibility.

## Impact
- Affected specs: `subscriber-node-builder`
- Affected code: `src/nodes/builder.rs`
