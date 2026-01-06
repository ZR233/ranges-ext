# ranges-ext

An efficient range/interval set data structure designed for `no_std` environments, with support for metadata, smart merging, and interval splitting.

## Core Features

- **Trait-based Design** - Support custom interval types via `RangeInfo` trait
- **Smart Merging** - Automatically merge overlapping or adjacent intervals with the same kind
- **Metadata Support** - Each interval can carry custom metadata (kind)
- **Overwrite Control** - Precise control over which intervals can be overwritten
- **Dual Mode Support** - heapless mode (stack-allocated) and alloc mode (heap-allocated)
- **Interval Splitting** - Remove operations can automatically split existing intervals
- **Zero-copy Iteration** - Efficient interval traversal
- **`no_std` Compatible** - Suitable for embedded and bare-metal environments

## Installation

### Basic Installation (heapless mode)

```toml
[dependencies]
ranges-ext = "0.5"
```

### Enable Alloc Feature (optional)

```toml
[dependencies]
ranges-ext = { version = "0.5", features = ["alloc"] }
```

This library is `#![no_std]` by default and can be used directly in embedded environments. Enable the `alloc` feature to use dynamic capacity mode in standard environments.

## Quick Start

### Heapless Mode (suitable for no_std environments)

```rust
use ranges_ext::{RangeInfo, RangeVecOps};
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MyRange {
    range: Range<i32>,
    kind: &'static str,
    overwritable: bool,
}

impl Default for MyRange {
    fn default() -> Self {
        Self {
            range: 0..0,
            kind: "",
            overwritable: false,
        }
    }
}

impl RangeInfo for MyRange {
    type Kind = &'static str;
    type Type = i32;

    fn range(&self) -> Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> Self::Kind {
        self.kind
    }

    fn overwritable(&self) -> bool {
        self.overwritable
    }

    fn clone_with_range(&self, range: Range<Self::Type>) -> Self {
        Self {
            range,
            kind: self.kind,
            overwritable: self.overwritable,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary buffer (required for heapless mode)
    let mut temp_buffer = [0u8; 1024];

    // Use heapless::Vec as container
    let mut set: heapless::Vec<MyRange, 16> = heapless::Vec::new();

    // Add intervals (adjacent/overlapping intervals with same kind will auto-merge)
    set.merge_add(MyRange {
        range: 10..20,
        kind: "A",
        overwritable: true,
    }, &mut temp_buffer)?;

    set.merge_add(MyRange {
        range: 30..40,
        kind: "A",
        overwritable: true,
    }, &mut temp_buffer)?;

    set.merge_add(MyRange {
        range: 15..35,
        kind: "A",
        overwritable: true,
    }, &mut temp_buffer)?;

    // Three intervals merge into one
    assert_eq!(set.len(), 1);
    assert_eq!(set.as_slice()[0].range(), 10..40);

    // Query
    assert!(set.contains_point(10));
    assert!(!set.contains_point(45));

    Ok(())
}
```

### Alloc Mode (suitable for standard environments)

```rust
use ranges_ext::{RangeInfo, RangeVecAllocOps};

// [Same MyRange definition as above]

#[cfg(feature = "alloc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use alloc::vec::Vec as container (no temporary buffer needed)
    let mut set: alloc::vec::Vec<MyRange> = alloc::vec::Vec::new();

    // Add intervals (no temporary buffer needed)
    set.merge_add(MyRange {
        range: 10..20,
        kind: "A",
        overwritable: true,
    })?;

    set.merge_add(MyRange {
        range: 15..35,
        kind: "A",
        overwritable: true,
    })?;

    // Auto-merge
    assert_eq!(set.len(), 1);

    Ok(())
}
```

### Simple Intervals Without Metadata

If you don't need metadata, you can use the unit type `()`:

```rust
use ranges_ext::{RangeInfo, RangeVecOps};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SimpleRange {
    range: core::ops::Range<i32>,
    overwritable: bool,
}

impl Default for SimpleRange {
    fn default() -> Self {
        Self {
            range: 0..0,
            overwritable: false,
        }
    }
}

impl RangeInfo for SimpleRange {
    type Kind = ();
    type Type = i32;

    fn range(&self) -> core::ops::Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> Self::Kind {
        ()
    }

    fn overwritable(&self) -> bool {
        self.overwritable
    }

    fn clone_with_range(&self, range: core::ops::Range<Self::Type>) -> Self {
        Self {
            range,
            overwritable: self.overwritable,
        }
    }
}

let mut temp_buffer = [0u8; 1024];
let mut set: heapless::Vec<SimpleRange, 16> = heapless::Vec::new();

set.merge_add(SimpleRange {
    range: 10..20,
    overwritable: true,
}, &mut temp_buffer)?;
```

## Core Concepts

### RangeInfo Trait

`RangeInfo` is the core trait that defines the requirements for interval types:

```rust
pub trait RangeInfo: Debug + Clone + Sized + Default {
    type Kind: Debug + Eq + Clone;  // Metadata type
    type Type: Ord + Copy;           // Interval value type

    fn range(&self) -> Range<Self::Type>;      // Get interval
    fn kind(&self) -> Self::Kind;              // Get metadata (owned)
    fn overwritable(&self) -> bool;            // Whether it can be overwritten
    fn clone_with_range(&self, range: Range<Self::Type>) -> Self;  // Clone with new range
}
```

**Important Change (0.5.0)**: The `kind()` method now returns the owned type `Self::Kind` instead of a reference `&Self::Kind`.

### Two Operation Modes

#### RangeVecOps (Heapless Mode)

Provides interval operations for fixed-capacity vectors (like `heapless::Vec<T, N>`):

- **Requires temporary buffer**: All methods need a `&mut [u8]` for temporary storage
- **Use cases**: no_std environments, embedded systems, deterministic memory usage

```rust
fn merge_add(&mut self, new_info: T, temp: &mut [u8]) -> Result<(), RangeError<T>>;
fn merge_remove(&mut self, range: Range<T::Type>, temp: &mut [u8]) -> Result<(), RangeError<T>>;
fn merge_extend<I>(&mut self, ranges: I, temp: &mut [u8]) -> Result<(), RangeError<T>>
where I: IntoIterator<Item = T>;
fn contains_point(&self, value: T::Type) -> bool;
```

#### RangeVecAllocOps (Alloc Mode)

Provides interval operations for dynamic vectors (`alloc::vec::Vec<T>`):

- **No temporary buffer needed**: Internally manages temporary storage
- **Use cases**: Standard environments, dynamic capacity needed

```rust
fn merge_add(&mut self, new_info: T) -> Result<(), RangeError<T>>;
fn merge_remove(&mut self, range: Range<T::Type>) -> Result<(), RangeError<T>>;
fn merge_extend<I>(&mut self, ranges: I) -> Result<(), RangeError<T>>
where I: IntoIterator<Item = T>;
fn contains_point(&self, value: T::Type) -> bool;
```

### Kind System

**Kind** is metadata for each interval, used to:

1. **Control merging behavior**: Only adjacent intervals with the same kind will merge
2. **Distinguish interval types**: e.g., "read-only", "read-write", "reserved"
3. **Implement business logic**: e.g., different memory region types, permission levels

Example:

```rust
// Same kind, will merge
set.merge_add(MyRange::new(0..10, "A", true), &mut temp)?;
set.merge_add(MyRange::new(10..20, "A", true), &mut temp)?;
// Result: [0, 20) kind="A"

// Different kinds, won't merge
set.merge_add(MyRange::new(0..10, "A", true), &mut temp)?;
set.merge_add(MyRange::new(10..20, "B", true), &mut temp)?;
// Result: [0, 10) kind="A", [10, 20) kind="B"
```

### Temporary Buffer Explanation

In heapless mode, `merge_add` and `merge_remove` operations require a temporary buffer.

**Why is it needed?**

- Interval splitting and merging need temporary storage for intermediate results
- heapless::Vec cannot dynamically expand during operations
- Using byte buffers avoids additional generic parameters

**How to calculate size?**

```rust
// Temporary buffer size = element size × expected max number of elements
let elem_size = std::mem::size_of::<MyRange>();
let max_elements = 128;  // Adjust based on actual needs
let mut temp_buffer = [0u8; elem_size * max_elements];

// More conservative calculation (considering alignment and splitting operations)
let mut temp_buffer = [0u8; elem_size * max_elements * 2];
```

**Best practices:**

- Reserve sufficient space: At least enough to store all current elements
- Reusable: The same buffer can be used for multiple operations
- Can be static: Suitable for global singleton scenarios

## Usage Guide

### Interval Overwrite Control

```rust
let mut set: heapless::Vec<MyRange, 16> = heapless::Vec::new();
let mut temp = [0u8; 1024];

// Add a non-overwritable interval
set.merge_add(MyRange {
    range: 10..30,
    kind: "protected",
    overwritable: false,  // Not overwritable
}, &mut temp)?;

// Attempting to add a conflicting interval will fail
let result = set.merge_add(MyRange {
    range: 20..40,
    kind: "new",
    overwritable: true,
}, &mut temp);
assert!(matches!(result, Err(RangeError::Conflict { .. })));

// Overwritable intervals can be replaced
set.merge_add(MyRange {
    range: 5..15,
    kind: "overwritable",
    overwritable: true,  // Overwritable
}, &mut temp)?;

set.merge_add(MyRange {
    range: 10..25,
    kind: "replacer",
    overwritable: true,
}, &mut temp)?;
// [5, 15) is partially overlapped and merged by [10, 25)
```

### Interval Splitting

```rust
let mut set: heapless::Vec<MyRange, 16> = heapless::Vec::new();
let mut temp = [0u8; 1024];

set.merge_add(MyRange::new(10..40, "A", true), &mut temp)?;
// Current: [10, 40) kind="A"

// Remove middle part
set.merge_remove(20..30, &mut temp)?;
// Result: [10, 20) kind="A", [30, 40) kind="A"
assert_eq!(set.len(), 2);
```

### Batch Operations

```rust
let ranges = vec![
    MyRange::new(0..10, "A", true),
    MyRange::new(10..20, "A", true),
    MyRange::new(20..30, "B", true),
];

// heapless mode
set.merge_extend(ranges, &mut temp)?;

// alloc mode
set.merge_extend(ranges)?;
```

## Error Handling

```rust
pub enum RangeError<T>
where
    T: RangeInfo,
{
    /// Insufficient capacity (heapless mode only)
    Capacity,

    /// Interval conflict: attempting to overwrite a non-overwritable interval
    Conflict {
        new: T,        // Newly added interval
        existing: T,   // Existing conflicting interval
    },
}
```

Example:

```rust
match set.merge_add(new_range, &mut temp_buffer) {
    Ok(()) => println!("Add successful"),
    Err(RangeError::Capacity) => println!("Insufficient capacity"),
    Err(RangeError::Conflict { new, existing }) => {
        println!("Conflict: new interval {:?} conflicts with {:?}", new, existing);
    }
}
```

## Example Code

The project includes the following examples (located in the `examples/` directory):

### debug_demo.rs - Basic Debugging

Demonstrates basic interval merging and iteration.

Run:

```bash
cargo run --example debug_demo
```

### key_demo.rs - Kind Demonstration

Shows how intervals with different kinds interact.

Run:

```bash
cargo run --example key_demo
```

### overlap_demo.rs - Overwrite and Splitting

Demonstrates various scenarios of interval overwriting and splitting.

Run:

```bash
cargo run --example overlap_demo
```

### slicevec_demo.rs - Temporary Buffer

Shows how to use temporary buffers.

Run:

```bash
cargo run --example slicevec_demo
```

## Mode Selection

### Heapless Mode

**Pros:**

- Stack-allocated, no heap fragmentation
- Predictable memory usage
- Suitable for no_std environments
- Compile-time determined capacity

**Cons:**

- Requires temporary buffer
- Fixed capacity, may overflow
- Manual buffer size management needed

**Use cases:**

- Embedded systems
- Bare-metal programs
- Deterministic memory management required
- Predictable number of intervals

### Alloc Mode

**Pros:**

- No temporary buffer needed
- Dynamic capacity, won't overflow
- Simpler API

**Cons:**

- Requires heap allocator
- Potential memory fragmentation
- Not suitable for strict no_std environments

**Use cases:**

- Standard environments
- Unpredictable number of intervals
- Development convenience prioritized

## Performance and Limitations

### Time Complexity

- **Add interval** (`merge_add`): O(n) - needs to traverse existing intervals
- **Remove interval** (`merge_remove`): O(n) - needs to traverse and split
- **Query contains** (`contains_point`): O(log n) - uses binary search
- **Iteration** (`iter`): O(n) - zero-copy, just iteration

### Space Complexity

- **heapless mode**: O(N) - N is compile-time specified capacity
- **alloc mode**: O(n) - n is actual number of stored intervals
- **Temporary buffer**: O(n) - additional space needed for heapless mode

### Limitations

1. **Capacity limitation (heapless mode)**

   - Exceeding capacity returns `RangeError::Capacity`
   - Need to estimate maximum number of intervals

2. **Type requirements**

   - `RangeInfo::Type` must implement `Ord + Copy`
   - `RangeInfo::Kind` must implement `Debug + Eq + Clone`

3. **Interval semantics**
   - Uses half-open intervals `[start, end)`
   - Intervals with `start >= end` are ignored

## FAQ

### Q: How to calculate the temporary buffer size?

**A:** Buffer size depends on interval type size and expected maximum number of elements:

```rust
let elem_size = std::mem::size_of::<MyRange>();
let max_elements = 128;  // Expected max number of intervals
let mut temp_buffer = [0u8; elem_size * max_elements];

// More conservative calculation (considering alignment and split operations)
let mut temp_buffer = [0u8; elem_size * max_elements * 2];
```

### Q: Why do only intervals with the same kind merge?

**A:** This design supports finer-grained interval management:

```rust
// Scenario: memory region management
// May need to distinguish "read-only", "read-write", "reserved" regions
// Even if adjacent, they should not merge

set.merge_add(MemoryRange::new(0x1000..0x2000, "read-only", true), &mut temp)?;
set.merge_add(MemoryRange::new(0x2000..0x3000, "read-write", true), &mut temp)?;
// Result keeps two separate intervals
```

### Q: How to use in strict no_std environments?

**A:** Use heapless mode with stack-allocated buffer:

```rust
#![no_std]

#[cfg(test)]
mod tests {
    use ranges_ext::{RangeInfo, RangeVecOps};

    #[test]
    fn test_heapless() {
        let mut temp_buffer = [0u8; 1024];
        let mut set: heapless::Vec<MyRange, 16> = heapless::Vec::new();
        // ... test code
    }
}
```

### Q: What should I pay attention to when upgrading from 0.2.x to 0.5.0?

**A:** Major changes:

1. **RangeInfo::kind() return type changed**

   ```rust
   // Old version
   fn kind(&self) -> &Self::Kind;

   // New version (0.5.0)
   fn kind(&self) -> Self::Kind;
   ```

2. **RangeSet struct no longer provided**

   - Directly use `heapless::Vec<T, N>` or `alloc::vec::Vec<T>`
   - Gain interval operations through `RangeVecOps` or `RangeVecAllocOps` traits

3. **API renamed**
   - `add()` → `merge_add()`
   - `remove()` → `merge_remove()`
   - `extend()` → `merge_extend()`
   - `contains()` → `contains_point()`

### Q: How to handle interval conflicts?

**A:** Use the `overwritable` field to control:

```rust
// Protect critical intervals
set.merge_add(MyRange::new(0..1000, "critical", false), &mut temp)?;

// Attempting to overwrite will fail
match set.merge_add(MyRange::new(500..1500, "new", true), &mut temp) {
    Err(RangeError::Conflict { new, existing }) => {
        eprintln!("Cannot overwrite critical interval: {:?}", existing);
    }
    _ => {}
}
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.

## License

Dual licensed under MIT or Apache-2.0, you may choose either.
