# ranges-ext

A simple and efficient range/interval set data structure designed for `no_std` environments.

- Uses half-open range semantics: `[start, end)` (i.e. `start..end`)
- **Automatically merges** overlapping or adjacent ranges
- Checks whether a value is contained in any range
- Removes a range by subtracting the intersection; can split an existing range into two
- Fixed-size capacity using `heapless::Vec` for stack allocation

## Installation

```toml
[dependencies]
ranges-ext = "0.1"
```

This crate is `#![no_std]` and uses `heapless::Vec` for storage.

## Usage

```rust
use ranges_ext::RangeSet;

// Create a new RangeSet with default capacity (128)
let mut set = RangeSet::<i32>::new();

// Add ranges (automatically normalized/merged)
set.add(10..20);
set.add(30..40);
set.add(15..35);  // Overlaps with previous ranges, will be merged
assert_eq!(set.as_slice(), &[10..40]);

// Query
assert!(set.contains(10));
assert!(!set.contains(40));

// Remove intersection (may trigger splitting)
set.clear();
set.add(10..50);
set.remove(20..30);  // Splits the range into two
assert_eq!(set.as_slice(), &[10..20, 30..50]);

// Iterate over ranges
for range in set.iter() {
    println!("[{}, {})", range.start, range.end);
}

// Batch add multiple ranges
set.extend([10..15, 20..25, 30..35]);
```

## Custom Capacity

You can specify a custom capacity using the const generic parameter:

```rust
// Create a RangeSet with capacity for 16 ranges
let mut set: RangeSet<i32, 16> = RangeSet::new();
set.add(1..10);
set.add(20..30);
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
