# Improvements

This document tracks potential improvements identified during code review.

## Range Arithmetic Overflow/Underflow

**Location:** `src/lib.rs`, `query_range()` method.

**Problem:** The range bound conversion has potential arithmetic issues:

```rust
Bound::Excluded(&start) => start as u32 + 1,  // overflow if start == u32::MAX
// ...
Bound::Included(&end) => end as u32 - range_start + 1,  // underflow if end < range_start
Bound::Excluded(&end) => end as u32 - range_start,       // underflow if end < range_start
```

Additionally, `usize` to `u32` truncation can occur silently on 64-bit systems when values exceed `u32::MAX`.

**Recommendation:** Use `saturating_add`, `saturating_sub`, and `try_into().unwrap_or(u32::MAX)`.

## Unnecessary Clone

**Location:** `src/lib.rs:343`

**Problem:** `full_path_name()` returns `Result<PathBuf>`, so the `path` variable is already owned. The `.clone()` allocates unnecessarily.

```rust
path: path.clone(),  // unnecessary, path is already owned
```

**Recommendation:** Move `path` directly instead of cloning.

## No Tests

**Problem:** The crate has no unit tests. Key untested areas:

- `convert_filetime` edge cases (dates before Unix epoch, overflow)
- Range bound conversions in `query_range`
- Metadata conditional extraction logic
- Error handling paths

**Recommendation:** Add unit tests for pure functions and integration tests for SDK interactions.

## Opaque `attributes` Field

**Problem:** The `attributes` field is `Option<u32>`, requiring users to know Windows API constants (`FILE_ATTRIBUTE_HIDDEN = 0x2`, etc.) to interpret it.

**Recommendation:** Consider a `FileAttributes` bitflags type with named constants, or helper methods like `is_hidden()`, `is_readonly()`.

## Glob Import of `everything_sdk`

**Note:** The `use everything_sdk::*` import brings `everything_sdk::Result` into scope, which shadows `std::result::Result`. This works fine since the crate only uses the SDK's `Result` internally, but future maintenance should be aware of this if `std::Result` is ever needed.
