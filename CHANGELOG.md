# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Current Version

### Breaking Changes

- 🔥 **Breaking**: `RangeInfo::kind()` method now returns owned type `Self::Kind` instead of reference `&Self::Kind`
- 🔥 **Breaking**: Removed `RangeSet` struct, changed to trait-based system
- API renamed:
  - `add()` → `merge_add()`
  - `remove()` → `merge_remove()`
  - `extend()` → `merge_extend()`
  - `contains()` → `contains_point()`

### Added

- ✨ New `RangeVecOps` trait for heapless mode (stack-allocated containers)
- ✨ New `RangeVecAllocOps` trait for alloc mode (heap-allocated containers)
- ✨ New `alloc` feature support for using dynamic containers
- ✨ Comprehensive documentation and examples

### Changed

- 📝 Refactored all examples and test cases to use heapless vectors
- 📝 Updated README with detailed trait system documentation
- 📝 Added migration guide for upgrading from 0.2.x

### Migration Guide from 0.2.x to 0.5.0

#### 1. Update `RangeInfo::kind()` implementation

**Old version (0.2.x):**

```rust
fn kind(&self) -> &Self::Kind {
    &self.kind
}
```

**New version (0.5.0):**

```rust
fn kind(&self) -> Self::Kind {
    self.kind
}
```

#### 2. Replace `RangeSet` with container types

**Old version (0.2.x):**

```rust
use ranges_ext::{RangeSet, RangeInfo};
let mut set = RangeSet::<MyRange>::new();
set.add(range)?;
```

**New version (0.5.0) - heapless mode:**

```rust
use ranges_ext::{RangeInfo, RangeVecOps};
let mut set: heapless::Vec<MyRange, 16> = heapless::Vec::new();
set.merge_add(range, &mut temp_buffer)?;
```

**New version (0.5.0) - alloc mode:**

```rust
use ranges_ext::{RangeInfo, RangeVecAllocOps};
let mut set: alloc::vec::Vec<MyRange> = alloc::vec::Vec::new();
set.merge_add(range)?;
```

#### 3. Update method calls

- `set.add(range)` → `set.merge_add(range, &mut temp)` (heapless) or `set.merge_add(range)` (alloc)
- `set.remove(range)` → `set.merge_remove(range, &mut temp)` (heapless) or `set.merge_remove(range)` (alloc)
- `set.extend(ranges)` → `set.merge_extend(ranges, &mut temp)` (heapless) or `set.merge_extend(ranges)` (alloc)
- `set.contains(value)` → `set.contains_point(value)`

## [0.4.x] - Previous Versions

### Internal

- Internal refactoring to prepare trait system
- Performance optimizations

## [0.3.x] - Previous Versions

### Changed

- Optimized `RangeInfo` trait
- Improved documentation

## [0.2.x] - Initial Release

### Added

- Initial version with `RangeSet` struct
- Basic interval operations (add, remove, contains)
- Support for metadata (kind)
- Interval overwrite control
- Automatic merging of adjacent/overlapping intervals

[0.5.0]: https://github.com/ZR233/ranges-ext/releases/tag/v0.5.0
[0.4.x]: https://github.com/ZR233/ranges-ext/compare/v0.3.0...v0.4.0
[0.3.x]: https://github.com/ZR233/ranges-ext/compare/v0.2.0...v0.3.0
[0.2.x]: https://github.com/ZR233/ranges-ext/releases/tag/v0.2.0
