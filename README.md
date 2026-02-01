# everywhere

A high-level, ergonomic Rust wrapper around the [Everything SDK](https://www.voidtools.com/support/everything/sdk/) for fast Windows file search.

## Requirements

- Windows (x86 or x64);
- [Everything](https://www.voidtools.com/) must be installed and running.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
everywhere = "0.1"
```

Or run `cargo add everywhere`.

## Quick Start

```rust
use everywhere::*;

// Simple search using Everything's native syntax
let results = search("*.rs").query_all();
for item in results {
    println!("{}", item.path.display());
}

// Search with options, limited to first 100 results
let results = search("foo bar")
    .match_case(true)
    .sort_by(SortKey::DateModified, SortOrder::Descending)
    .request_metadata(ItemMetadata::SIZE | ItemMetadata::DATE_MODIFIED)
    .query_range(..100);

for item in results {
    println!("{}: {:?} bytes", item.path.display(), item.size);
}

// Regex search
let results = search_regex(r"src[/\\].*\.rs$").query_all();
```

## Features

- **Simple and Fluent API**: Start with `search("pattern").query_all()` and combine various options by chaining `.match_case()`, `.sort_by()`, etc.
- **Atomic queries**: Results are fully owned `Vec<Item>` — no lifetime guards, no invalidation, store and pass freely.
- **Optional metadata**: Request file size, dates, and attributes only when needed.

## Design Decisions

The Everything SDK uses global state — starting a new query invalidates previous results. Other wrappers solve this with lifetime-guarded iterators that borrow the global state.

This crate takes a different approach: **queries are atomic**. When you call `.query_all()` or `.query_range()`, all results are retrieved into an owned `Vec<Item>`. You can store results, pass them across threads, and start new queries without invalidating old results.

## Caveats

**No snapshot isolation:** The Everything index is live. If files change between queries, consecutive `query_range` calls (e.g., `0..100` then `100..200`) may have gaps, overlaps, or inconsistent ordering. This limitation is inherent to the Everything indexing system. To get consistent results, fetch everything you need in a single call to `.query_all()` or `.query_range()`.

## Documentation

- [API Reference](https://docs.rs/everywhere) — Full API documentation
- [Design Documentation](docs/README.md) — Design principles and decisions

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
