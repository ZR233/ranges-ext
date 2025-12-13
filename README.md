# ranges-ext

An efficient range/interval set data structure designed for `no_std` environments.

- Uses half-open interval semantics: `[start, end)` (i.e., `start..end`)
- **Automatic merging** of overlapping or adjacent intervals
- Supports querying whether a value falls within any interval
- Supports interval removal, removing intersections from the set; splits existing intervals when necessary
- Uses `heapless::Vec` for fixed-capacity storage with stack allocation support
- **Supports metadata (kind)**: Each interval can carry custom metadata
- **Supports overwrite control**: Specify whether intervals can be overwritten by other intervals

## Installation

```toml
[dependencies]
ranges-ext = "0.2"
```

This library is `#![no_std]` and uses `heapless::Vec` for storage.

## Quick Start

```rust
use ranges_ext::{RangeSet, RangeInfo};

// First, define a struct that implements the RangeInfo trait
#[derive(Clone, Debug, PartialEq, Eq)]
struct MyRange {
    range: core::ops::Range<i32>,
    kind: &'static str,      // metadata
    overwritable: bool,     // whether overwritable
}

impl MyRange {
    fn new(range: core::ops::Range<i32>, kind: &'static str, overwritable: bool) -> Self {
        Self { range, kind, overwritable }
    }
}

impl RangeInfo for MyRange {
    type Kind = &'static str;
    type Type = i32;

    fn range(&self) -> core::ops::Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> &Self::Kind {
        &self.kind
    }

    fn overwritable(&self) -> bool {
        self.overwritable
    }

    fn clone_with_range(&self, range: core::ops::Range<Self::Type>) -> Self {
        Self {
            range,
            kind: self.kind,
            overwritable: self.overwritable,
        }
    }
}

// Create RangeSet
let mut set = RangeSet::<MyRange>::new();

// Add intervals (intervals with the same kind that are adjacent or overlapping will automatically merge)
set.add(MyRange::new(10..20, "A", true))?;
set.add(MyRange::new(30..40, "A", true))?;
set.add(MyRange::new(15..35, "A", true))?;  // Overlaps with the first two intervals, will merge
assert_eq!(set.len(), 1);
assert_eq!(set.as_slice()[0].range(), &(10..40));
assert_eq!(set.as_slice()[0].kind(), &"A");

// Add intervals with different kinds
set.add(MyRange::new(35..45, "B", true))?;
assert_eq!(set.len(), 2);

// Query
assert!(set.contains(10));
assert!(set.contains(39));
assert!(!set.contains(45));

// Remove intervals (preserves intersections with other intervals)
set.remove(20..30)?;
assert_eq!(set.len(), 2);

// Iterator
for info in set.iter() {
    println!("[{}, {}) kind={}", info.range().start, info.range().end, info.kind());
}
```

## Basic Usage (No Metadata)

If you don't need metadata, you can use a simpler struct:

```rust
use ranges_ext::{RangeSet, RangeInfo};

#[derive(Clone, Debug)]
struct SimpleRange {
    range: core::ops::Range<i32>,
    overwritable: bool,
}

impl SimpleRange {
    fn new(range: core::ops::Range<i32>, overwritable: bool) -> Self {
        Self { range, overwritable }
    }
}

impl RangeInfo for SimpleRange {
    type Kind = ();
    type Type = i32;

    fn range(&self) -> core::ops::Range<Self::Type> {
        self.range.clone()
    }

    fn kind(&self) -> &Self::Kind {
        &()
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

let mut set = RangeSet::<SimpleRange>::new();
set.add(SimpleRange::new(10..20, true))?;
set.add(SimpleRange::new(30..40, true))?;
```

## Custom Capacity

You can specify custom capacity through const generic parameters:

```rust
// Create a RangeSet with capacity 16
let mut set: RangeSet<MyRange, 16> = RangeSet::new();
```

## Core Trait: RangeInfo

```rust
pub trait RangeInfo: Debug + Clone + Sized {
    type Kind: Debug + Eq + Clone;
    type Type: Ord + Copy;

    fn range(&self) -> Range<Self::Type>;
    fn kind(&self) -> &Self::Kind;
    fn overwritable(&self) -> bool;
    fn clone_with_range(&self, range: Range<Self::Type>) -> Self;
}
```

## API Reference

### Constructors

- `RangeSet<T, C>::new()` - Create an empty set
- `RangeSet<T, C>::default()` - Create an empty set through the Default trait

### Interval Operations

- `add(info)` - Add interval, automatically merge overlapping/adjacent intervals of the same kind. Returns `Result<(), RangeError>`
- `extend(ranges)` - Batch add multiple intervals. Returns `Result<(), RangeError>`
- `remove(range)` - Remove interval intersections; may trigger interval splitting. Returns `Result<(), RangeError>`

### Query Methods

- `contains(value)` - Check if a value is contained by any interval
- `is_empty()` - Check if the set is empty
- `len()` - Return the number of merged intervals

### Access Methods

- `as_slice()` - Return a normalized interval slice (sorted, merged, non-overlapping)
- `iter()` - Return a normalized interval iterator (zero-copy)

### Other Methods

- `clear()` - Clear the set

## Advanced Features

### Interval Overwrite Control

```rust
// Add a non-overwritable interval
set.add(MyRange::new(10..30, "protected", false))?;

// Attempting to add a conflicting interval will return an error
let result = set.add(MyRange::new(20..40, "new", true));
assert!(result.is_err()); // Returns RangeError::Conflict

// Overwritable intervals can be replaced
set.add(MyRange::new(5..15, "overwritable", true))?; // Overwritable
set.add(MyRange::new(10..25, "replacer", true))?; // Will overwrite overlapping parts
```

### Error Handling

```rust
match set.add(range) {
    Ok(()) => println!("Add successful"),
    Err(RangeError::Capacity) => println!("Insufficient capacity"),
    Err(RangeError::Conflict { new, existing }) => {
        println!("Interval conflict: {:?} conflicts with {:?}", new, existing);
    }
}
```

## Features

- **`no_std` compatible**: Suitable for embedded and bare-metal environments
- **Zero-copy iteration**: `iter()` method returns interval references, avoiding unnecessary copying
- **Smart merging**: Adjacent/overlapping intervals with the same kind automatically merge
- **Efficient implementation**: Uses binary search to optimize insertion and query performance
- **Flexible metadata**: Supports arbitrary type metadata
- **Overwrite control**: Precise control over which intervals can be overwritten by other intervals

## Examples

See the `examples/` directory for more examples:

- `debug_demo.rs` - Basic debugging example
- `key_demo.rs` - Example using different kinds
- `overlap_demo.rs` - Detailed demonstration of interval overwriting and merging

## License

Dual licensed under MIT or Apache-2.0, you may choose either.