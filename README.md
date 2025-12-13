# ranges-ext

`ranges-ext` is a range/interval set data structure designed for `no_std + alloc` environments.

- Uses half-open range semantics: `[start, end)` (i.e. `start..end`)
- Merges overlapping or adjacent ranges on insertion
- Checks whether a value is contained in any range
- Removes a range by subtracting the intersection; can split an existing range into two

## Installation

```toml
[dependencies]
ranges-ext = "0.1"
```

This crate is `#![no_std]` by default and depends on `alloc`.

## Usage

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
set.add_range(r(15, 35));
assert_eq!(set.as_slice(), &[r(10, 40)]);

// Query
assert!(set.contains(10));
assert!(!set.contains(40));

// Remove intersection (may split)
set.clear();
set.add_range(r(10, 50));
set.remove_range(r(20, 30));
assert_eq!(set.as_slice(), &[r(10, 20), r(30, 50)]);
```

## API (brief)

- `RangeSet<T>::add_range(range)`: insert and merge (empty ranges where `start >= end` are ignored)
- `RangeSet<T>::contains(value)`: containment test
- `RangeSet<T>::remove_range(range)`: subtract intersection; may split
- `RangeSet<T>::as_slice()`: view normalized ranges (sorted, non-overlapping)

## License

Licensed under either MIT or Apache-2.0, at your option.
