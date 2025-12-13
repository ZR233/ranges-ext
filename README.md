# ranges-ext

A range/interval set data structure designed for `no_std + alloc` environments.

- Uses half-open range semantics: `[start, end)` (i.e. `start..end`)
- **Automatically merges** overlapping or adjacent ranges
- Supports **metadata** attached to each range
- Checks whether a value is contained in any range
- Removes a range by subtracting the intersection; can split an existing range into two
- Supports **multiple output formats** (Debug, Display, hexadecimal, binary, octal, etc.)

## Installation

```toml
[dependencies]
ranges-ext = "0.1"
```

This crate is `#![no_std]` by default and depends on `alloc`.

## Basic Usage

```rust
use core::ops::Range;
use ranges_ext::RangeSet;

fn r(start: i32, end: i32) -> Range<i32> {
    start..end
}

let mut set = RangeSet::<i32>::new();

// Add ranges (automatically normalized/merged)
set.add_range(r(10, 20));
set.add_range(r(30, 40));
set.add_range(r(15, 35));  // Overlaps with previous ranges, will be merged
assert_eq!(set.as_slice(), &[r(10, 40)]);

// Query
assert!(set.contains(10));
assert!(!set.contains(40));

// Remove intersection (may trigger splitting)
set.clear();
set.add_range(r(10, 50));
set.remove_range(r(20, 30));
assert_eq!(set.as_slice(), &[r(10, 20), r(30, 50)]);
```

## Metadata Usage

The power of this library lies in its support for attaching metadata to each range:

```rust
use ranges_ext::RangeSet;

let mut set = RangeSet::<i32, &str>::new();

// Add ranges with metadata
set.add(10..20, "first");
set.add(15..25, "second");  // Overlaps with the previous one, will be merged
set.add(30..40, "third");

// Merged ranges preserve all original ranges and metadata
let elements = set.elements();
assert_eq!(elements.len(), 2);

// The first merged range [10,25) contains two original ranges
assert_eq!(elements[0].merged, 10..25);
assert_eq!(elements[0].originals.len(), 2);
assert_eq!(elements[0].originals[0].range, 10..20);
assert_eq!(elements[0].originals[0].meta, "first");
assert_eq!(elements[0].originals[1].range, 15..25);
assert_eq!(elements[0].originals[1].meta, "second");

// Adjacent ranges with same metadata are automatically merged
let mut set = RangeSet::<i32, &str>::new();
set.add(10..20, "same");
set.add(20..30, "same");  // Adjacent with same metadata
set.add(30..40, "same");  // Will be merged into one original range

assert_eq!(set.elements()[0].originals.len(), 1);
assert_eq!(set.elements()[0].originals[0].range, 10..40);
```

## Formatting Output

Supports multiple formatting options:

```rust
let mut set = RangeSet::<i32, &str>::new();
set.add(10..20, "first");
set.add(15..25, "second");

// Display format: only shows the merged range
println!("{}", set.elements()[0]);  // Output: [10..25)

// Debug format: shows detailed information
println!("{:?}", set.elements()[0]);
// Output: MergedRange { merged: [10..25), originals: [[10..20) → "first", [15..25) → "second"] }

// Hexadecimal format
println!("{:x}", set.elements()[0]);  // Output: [a..19)

// Binary format
println!("{:b}", set.elements()[0]);  // Output: [1010..11001)
```

## API Reference

### Constructors

- `RangeSet<T, M>::new()` - Create an empty set
- `RangeSet<T, M>::default()` - Create an empty set via Default trait

### Range Operations

- `add(range, meta)` - Add a range with metadata and merge
- `add_range(range)` - Add a range without metadata (only for `M = ()`)
- `extend(ranges)` - Batch add multiple ranges (only for `M = ()`)
- `remove_range(range)` - Remove range intersection; may trigger splitting

### Query Methods

- `contains(value)` - Check if value is contained in any range
- `is_empty()` - Check if the set is empty
- `len()` - Return the number of merged ranges

### Access Methods

- `elements()` - Return slice of internal elements (contains merged ranges and original lists)
- `as_slice()` - Return normalized range slice (only ranges, sorted, merged, non-overlapping)
- `iter()` - Return iterator of normalized ranges (zero-copy)

### Other Methods

- `clear()` - Clear the set

## Features

- **`no_std` compatible**: Suitable for embedded and bare-metal environments
- **Zero-copy iteration**: `iter()` method returns range references to avoid unnecessary copying
- **Smart merging**: Adjacent/overlapping original ranges with same metadata are automatically merged
- **Efficient implementation**: Uses binary search to optimize insertion and query performance
- **Flexible metadata**: Supports any type of metadata, can be `()` to indicate no metadata needed

## License

Licensed under either MIT or Apache-2.0, at your option.
