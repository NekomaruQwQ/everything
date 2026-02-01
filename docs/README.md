# Everywhere

A high-level, ergonomic Rust wrapper around the [Everything SDK](https://www.voidtools.com/support/everything/sdk/) for fast Windows file search.

## Goal

Everywhere provides a clean, idiomatic Rust API for querying the [Everything](https://www.voidtools.com/) file search engine. It abstracts away the global-state complexity of the underlying SDK while exposing only file-system-related concepts.

## Design Principles

### Simplicity Over Flexibility

The API exposes only concepts directly related to file system metadata. Everything-specific features that don't map cleanly to file system semantics are intentionally excluded:

- **Excluded:** Run count, date run, date recently changed, file lists, highlighted results
- **Included:** Size, creation/modification/access dates, attributes, path

Users who need Everything-specific features can use the lower-level [`everything-sdk`](https://crates.io/crates/everything-sdk) crate directly.

### Atomic Queries

The Everything SDK uses global state internally. Starting a new query invalidates previous results. Other wrappers handle this by returning results tied to a lifetime guard.

Everywhere takes a different approach: queries are **atomic**. When you call `.query_all()` or `.query_range()`:

1. The global state is locked
2. All search parameters are set
3. The query executes
4. **All requested results are copied into owned data structures**
5. The lock is released

The returned `Vec<Item>` is fully owned. You can store it indefinitely, pass it across threads, and execute new queries without invalidating previous results.

### Prefer In-Memory Processing Over IPC

The Everything SDK runs as a separate process. Each piece of data requested requires inter-process communication. Everywhere minimizes IPC by:

- **Always requesting full paths:** Rather than exposing options for file name, directory, or full path separately, Everywhere always requests the full path. Users extract components using Rust's standard `Path` methods (`file_name()`, `parent()`, `extension()`). Parsing an in-memory path is trivial compared to an additional IPC round-trip.

- **Not exposing extension as separate metadata:** The extension is trivially derivable from the path. Requesting it separately would add IPC overhead for no benefit.

### Graceful Error Handling

Everywhere prioritizes resilience over strictness:

- **Partial results over total failure:** If an individual item fails to load (e.g., path retrieval fails), it is skipped and logged rather than failing the entire query.
- **Logging over silent failure:** Errors are logged via the [`log`](https://crates.io/crates/log) crate. Users can attach any logger (e.g., `env_logger`, `tracing`) to see warnings.
- **Debug assertions for invariants:** SDK invariant violations (e.g., an item that is neither file, folder, nor volume) trigger `debug_assert!` in debug builds for early detection, while gracefully degrading in release builds.

## API Design

### Entry Points

Two functions create a `Search`:

```rust
use everywhere::{search, search_regex};

// Everything's native syntax (wildcards, boolean operators, filters)
let results = search("*.rs").query_all();

// Regular expression syntax
let results = search_regex(r"\.rs$").query_all();
```

The search syntax is determined by the entry point, avoiding the need for a separate `SearchSyntax` enum. Internally, this is just a `regex: bool` field.

### The `Search` Type

`Search` is a builder for configuring queries. It uses a fluent API with unprefixed setters (following `std::process::Command` conventions):

```rust
let results = search("*.rs")
    .match_case(true)
    .match_path(true)
    .sort_by(SortKey::DateModified, SortOrder::Descending)
    .request_metadata(ItemMetadata::SIZE | ItemMetadata::DATE_MODIFIED)
    .query_range(..100);  // Limit to first 100 results
```

**Design choices:**

- **Public fields:** `Search` fields are public, allowing direct construction if preferred. However, the fluent API is recommended.
- **Unprefixed setters:** Methods like `.match_case()` rather than `.with_match_case()` — shorter and conventional.
- **No getters:** Setters don't have corresponding getters. Use `#[derive(Debug)]` output for inspection if needed.
- **`query_` prefix on execution methods:** `.query_all()` and `.query_range()` are prefixed to signal that these are the blocking IPC operations.

### The `Item` Type

Search results are returned as `Vec<Item>`:

```rust
pub struct Item {
    pub path: PathBuf,
    pub item_type: ItemType,  // File, Folder, or Volume

    // Optional metadata (None if not requested)
    pub size: Option<u64>,
    pub date_created: Option<SystemTime>,
    pub date_modified: Option<SystemTime>,
    pub date_accessed: Option<SystemTime>,
    pub attributes: Option<u32>,
}
```

**Design choices:**

- **Single `path` field:** Full path only; use `Path` methods to extract components.
- **`Option<T>` for metadata:** `None` means the field was not requested via `request_metadata()`. If requested but retrieval fails, the error is logged and the field is `None`.
- **`Vec<Item>` return type:** Failed items are logged and skipped. The query never fails entirely due to individual item errors.

### Metadata Flags

`ItemMetadata` is a bitflags type for requesting optional metadata:

```rust
let results = search("*.rs")
    .request_metadata(ItemMetadata::SIZE | ItemMetadata::DATE_MODIFIED)
    .query_all();
```

The bit values are derived directly from `everything_sdk::RequestFlags` to ensure they stay in sync.

## Design Decisions

### Path Representation

`Item` contains a single `path: PathBuf` field rather than separate fields for file name, directory, and full path. This simplifies the API and avoids redundant data. Use `std::path::Path` methods to extract components:

```rust
let item: Item = /* ... */;
let file_name = item.path.file_name();    // Option<&OsStr>
let parent = item.path.parent();          // Option<&Path>
let extension = item.path.extension();    // Option<&OsStr>
```

### Metadata Fields

`SortKey` and `ItemMetadata` include only standard file system metadata:

| Field | Description |
|-------|-------------|
| `Size` | File size in bytes |
| `DateCreated` | NTFS creation timestamp |
| `DateModified` | NTFS last write timestamp |
| `DateAccessed` | NTFS last access timestamp |
| `Attributes` | NTFS file attributes |

The following Everything-specific fields are **intentionally excluded**:

| Field | Reason for Exclusion |
|-------|---------------------|
| `RunCount` | Everything-specific; tracks how often a file was opened via Everything |
| `DateRun` | Everything-specific; when the file was last opened via Everything |
| `DateRecentlyChanged` | Semantics are opaque; represents when Everything observed a change via the NTFS USN journal, not when the file actually changed |
| `FileListFileName` | Everything-specific file list feature |
| `Highlighted*` | Everything UI feature; not relevant to programmatic access |

### Search Syntax Representation

Rather than a `SearchSyntax` enum, the syntax is determined by the entry function:

- `search()` — Everything's native syntax
- `search_regex()` — Regular expression syntax

This avoids naming difficulties (Everything's syntax isn't a "standard") and makes the common case (native syntax) the default.

### Error Handling Strategy

| Error Type | Behavior |
|------------|----------|
| Path retrieval fails | Log error, skip item |
| Metadata retrieval fails | Log error, field is `None` |
| Item type indeterminate | `debug_assert!` in debug builds, skip item in release |
| Lock poisoning | Panic (indicates previous query panicked) |

This approach ensures users get partial results rather than total failure, while still surfacing issues via logging.

## Known Limitations

### No Snapshot Isolation for Paginated Queries

The Everything index is live and updates as the file system changes. When using `query_range()` for pagination:

- Consecutive ranges (e.g., `0..100` then `100..200`) may not be contiguous
- Files created between calls may appear in unexpected positions or be missed
- Files deleted between calls may cause gaps
- Files renamed between calls may appear in both ranges or neither

This is inherent to the Everything API, which provides no mechanism to freeze or snapshot the index.

**Recommendation:** If you need a consistent set of results, use `query_range(..limit)` to fetch everything in a single atomic operation.

### Global State

The underlying Everything SDK uses global state. Everywhere serializes access internally, but only one query can execute at a time within a process. Concurrent `query_*()` calls will block waiting for the lock.

## Dependencies

- [`everything-sdk`](https://crates.io/crates/everything-sdk) — Low-level Everything SDK bindings
- [`bitflags`](https://crates.io/crates/bitflags) — For `ItemMetadata` flags
- [`log`](https://crates.io/crates/log) — For error logging (users attach their own logger)
