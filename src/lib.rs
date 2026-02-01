#![cfg_attr(doc, doc = include_str!("../README.md"))]

use std::ffi::OsString;
use std::ops::Bound;
use std::ops::RangeBounds;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use everything_sdk::*;

/// Creates a new search with the given pattern using the Everything search
/// syntax.
///
/// This syntax supports wildcards (`*`, `?`), operators (`AND`, `OR`, `NOT`,
/// etc.) and a bunch of other features.
///
/// See [Searching - voidtools](https://www.voidtools.com/support/everything/searching/) for more details on the Everything search syntax.
pub fn search<S: Into<OsString>>(pattern: S) -> Search {
    Search {
        pattern: pattern.into(),
        ..Default::default()
    }
}

/// Creates a new search with the given regular expression pattern.
pub fn search_regex<S: Into<OsString>>(pattern: S) -> Search {
    Search {
        pattern: pattern.into(),
        regex: true,
        ..Default::default()
    }
}

/// Represents a search query to be executed against the Everything index.
///
/// Call [`Search::query_all`] or [`Search::query_range`] to execute the search
/// and retrieve results.
#[expect(clippy::struct_excessive_bools, reason = "Booleans are appropriate for configuring search options.")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct Search {
    /// The search pattern to use.
    pub pattern: OsString,

    /// Whether to use regular expressions for the search pattern. `false`
    /// by default.
    ///
    /// If `false`, the Everything search syntax is used. This syntax supports
    /// wildcards (`*`, `?`), operators (`AND`, `OR`, `NOT`, etc.) and a bunch
    /// of other features.
    ///
    /// See [Searching - voidtools](https://www.voidtools.com/support/everything/searching/) for more details on the Everything search syntax.
    pub regex: bool,

    /// Specifies whether the search is case-sensitive. `false` by default.
    pub match_case: bool,

    /// Specifies whether full path matching is enabled. `false` by default.
    pub match_path: bool,

    /// Specifies whether the search matches whole words only. `false` by default.
    pub match_whole_word: bool,

    /// Specifies how search results are sorted. See [`SortKey`] for details.
    ///
    /// The default sort key is [`SortKey::Name`].
    pub sort_key: SortKey,

    /// Specifies how search results are sorted. See [`SortOrder`] for details.
    ///
    /// The default sort order is [`SortOrder::Ascending`].
    pub sort_order: SortOrder,

    /// Specifies additional file system metadata to include in search results.
    /// See [`ItemMetadata`] for details.
    ///
    /// By default, no additional metadata is included.
    pub requested_metadata: ItemMetadata,
}

/// Specifies the order in which search results are sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum SortOrder {
    #[default] Ascending,
    Descending,
}

/// Specifies the key by which search results are sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum SortKey {
    #[default] Name,
    TypeName,
    Path,
    Size,
    Extension,
    DateCreated,
    DateModified,
    DateAccessed,
    Attributes,
}

/// Builder methods for [`Search`].
impl Search {
    /// Sets whether the search is case-sensitive.
    ///
    /// By default, searches are case-insensitive.
    #[must_use]
    #[inline]
    pub const fn match_case(mut self, case: bool) -> Self {
        self.match_case = case;
        self
    }

    /// Sets whether full path matching is enabled.
    ///
    /// By default, full path matching is disabled.
    #[must_use]
    #[inline]
    pub const fn match_path(mut self, path: bool) -> Self {
        self.match_path = path;
        self
    }

    /// Sets whether the search matches whole words only.
    ///
    /// By default, the search matches partial words.
    #[must_use]
    #[inline]
    pub const fn match_whole_word(mut self, whole_word: bool) -> Self {
        self.match_whole_word = whole_word;
        self
    }

    /// Sets the sort key and order for the search results.
    ///
    /// By default, results are sorted by name in ascending order.
    #[must_use]
    #[inline]
    pub const fn sort_by(mut self, key: SortKey, order: SortOrder) -> Self {
        self.sort_key = key;
        self.sort_order = order;
        self
    }

    /// Requests additional file system metadata to be included in search results.
    /// This method can be called multiple times and the requested metadata will
    /// be combined.
    ///
    /// By default, no additional metadata is included.
    ///
    /// See [`ItemMetadata`] for details.
    #[must_use]
    #[inline]
    pub fn request_metadata(mut self, metadata: ItemMetadata) -> Self {
        self.requested_metadata |= metadata;
        self
    }
}

/// Represents information about a file, folder or volume in the file system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    /// The full path of the item.
    pub path: PathBuf,

    /// The type of the item.
    pub item_type: ItemType,

    /// The size of the item in bytes if available.
    /// `None` if the field was not requested via [`Search::request_metadata`]
    /// or is not available. In the latter case, the error is logged.
    pub size: Option<u64>,

    /// The creation date of the item if available.
    /// `None` if the field was not requested via [`Search::request_metadata`]
    /// or is not available. In the latter case, the error is logged.
    pub date_created: Option<SystemTime>,

    /// The modification date of the item if available.
    /// `None` if the field was not requested via [`Search::request_metadata`]
    /// or is not available. In the latter case, the error is logged.
    pub date_modified: Option<SystemTime>,

    /// The access date of the item if available.
    /// `None` if the field was not requested via [`Search::request_metadata`]
    /// or is not available. In the latter case, the error is logged.
    pub date_accessed: Option<SystemTime>,

    /// The attributes of the item if available.
    /// `None` if the field was not requested via [`Search::request_metadata`]
    /// or is not available. In the latter case, the error is logged.
    pub attributes: Option<u32>,
}

/// Represents the type of the [`Item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    File,
    Folder,
    Volume,
}

bitflags::bitflags! {
    /// Specifies additional file system metadata to include in search results.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(Default)]
    pub struct ItemMetadata: u32 {
        const SIZE =
            RequestFlags::EVERYTHING_REQUEST_SIZE.bits();
        const DATE_CREATED =
            RequestFlags::EVERYTHING_REQUEST_DATE_CREATED.bits();
        const DATE_MODIFIED =
            RequestFlags::EVERYTHING_REQUEST_DATE_MODIFIED.bits();
        const DATE_ACCESSED =
            RequestFlags::EVERYTHING_REQUEST_DATE_ACCESSED.bits();
        const ATTRIBUTES =
            RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES.bits();
    }
}

/// Methods for executing the search.
impl Search {
    /// Executes the search and returns all matching items.
    ///
    /// This method returns all results without limit. For queries that may
    /// return a large number of results, consider using [`Self::query_range`]
    /// to limit results and avoid excessive memory usage.
    ///
    /// This method blocks until all results are retrieved. To avoid blocking,
    /// consider spawning a separate thread for the search.
    ///
    /// This method is equivalent to calling `query_range(..)`.
    #[must_use]
    #[inline]
    pub fn query_all(&self) -> Vec<Item> {
        self.query_range(..)
    }

    /// Executes the search and returns matching items within the specified
    /// range.
    ///
    /// Use this method to limit the number of results returned. For example,
    /// `query_range(..100)` returns the first 100 results, and
    /// `query_range(100..200)` returns results 100 through 199.
    ///
    /// For queries that may return a large number of results, it is strongly
    /// recommended to limit results to avoid excessive memory usage.
    ///
    /// This method blocks until the specified range of results is retrieved.
    /// To avoid blocking, consider spawning a separate thread for the search.
    ///
    /// # Caveats
    ///
    /// The Everything index is live and may change between queries. Consecutive
    /// calls (e.g., `query_range(0..100)` then `query_range(100..200)`) are not
    /// guaranteed to be consistent — files may be added, removed, or reordered
    /// between calls, causing gaps or overlaps. This limitation is inherent to
    /// the Everything indexing system. To get consistent results, fetch everything
    /// you need in a single call.
    #[must_use]
    pub fn query_range<R: RangeBounds<usize>>(&self, range: R) -> Vec<Item> {
        let range_start = match range.start_bound() {
            Bound::Included(&start) => start as u32,
            Bound::Excluded(&start) => start as u32 + 1,
            Bound::Unbounded => 0,
        };

        let range_len = match range.end_bound() {
            Bound::Included(&end) => end as u32 - range_start + 1,
            Bound::Excluded(&end) => end as u32 - range_start,
            Bound::Unbounded => u32::MAX,
        };

        let mut everything = everything_sdk::global().lock().unwrap();
        let mut searcher = everything.searcher();
        self.apply(&mut searcher);
        let result =
            searcher
                .set_offset(range_start)
                .set_max(range_len)
                .query();
        (0..result.num())
            .filter_map(|i| Item::from_result(self, &result, i))
            .collect()
    }

    fn apply(&self, searcher: &mut EverythingSearcher) {
        searcher
            .set_search(&self.pattern)
            .set_regex(self.regex)
            .set_match_case(self.match_case)
            .set_match_path(self.match_path)
            .set_match_whole_word(self.match_whole_word)
            .set_sort(
                convert_sort_type(
                    self.sort_key,
                    self.sort_order))
            .set_request_flags(
                RequestFlags::from_bits_truncate(
                    self.requested_metadata.bits()));
    }
}

impl Item {
    fn from_result(
        search: &Search,
        result: &EverythingResults,
        index: u32)
     -> Option<Self> {
        let item = result.at(index)?;

        let path =
            item.full_path_name(None).inspect_err(|err| {
                log::error!(concat!(
                    "Unable to retrieve the full path name of an item. ",
                    "Caused by the following error in the Everything SDK: {}"),
                    err);
            }).ok()?;

        let item_type_candidates = [
            item.is_file()
                .then_some(ItemType::File),
            item.is_folder()
                .then_some(ItemType::Folder),
            item.is_volume()
                .then_some(ItemType::Volume),
        ];

        debug_assert_eq!(
            item_type_candidates.iter().flatten().count(),
            1,
            concat!(
                "Encountering an item that is not exactly one of: file, folder, or volume. ",
                "This is likely a bug in the Everything SDK."));
        let item_type =
            item_type_candidates.iter().flatten().next().copied()?;

        Some(Self {
            path: path.clone(),
            item_type,
            size:
                get_metadata_from_item(
                    search,
                    &item,
                    &path,
                    ItemMetadata::SIZE,
                    EverythingItem::size),
            date_created:
                get_metadata_from_item(
                    search,
                    &item,
                    &path,
                    ItemMetadata::DATE_CREATED,
                    EverythingItem::date_created).map(convert_filetime),
            date_modified:
                get_metadata_from_item(
                    search,
                    &item,
                    &path,
                    ItemMetadata::DATE_MODIFIED,
                    EverythingItem::date_modified).map(convert_filetime),
            date_accessed:
                get_metadata_from_item(
                    search,
                    &item,
                    &path,
                    ItemMetadata::DATE_ACCESSED,
                    EverythingItem::date_accessed).map(convert_filetime),
            attributes:
                get_metadata_from_item(
                    search,
                    &item,
                    &path,
                    ItemMetadata::ATTRIBUTES,
                    EverythingItem::attributes),
        })
    }
}

/// Helper function to retrieve metadata from an [`EverythingItem`] if it was
/// requested in the [`Search`].
fn get_metadata_from_item<'a, T, F>(
    search: &Search,
    item: &'a EverythingItem<'a>,
    item_path: &Path,
    metadata_flag: ItemMetadata,
    metadata_getter: F)
    -> Option<T>
where
    F: FnOnce(&'a EverythingItem<'a>) -> Result<T>, {
    if search.requested_metadata.contains(metadata_flag) {
        match metadata_getter(item) {
            Ok(value) => Some(value),
            Err(err) => {
                log::error!(
                    concat!(
                        "Unable to retrieve requested metadata for {}. ",
                        "Caused by the following error in the Everything SDK: {}"),
                    item_path.display(),
                    err);
                None
            }
        }
    } else {
        None
    }
}

/// Converts a Windows FILETIME value to a [`SystemTime`].
///
/// FILETIME is the number of 100-nanosecond intervals since January 1, 1601.
fn convert_filetime(filetime: u64) -> SystemTime {
    // Difference between Windows epoch (1601-01-01) and Unix epoch (1970-01-01)
    // in 100-nanosecond intervals.
    const FILETIME_UNIX_DIFF: u64 = 116_444_736_000_000_000;

    let unix_100ns = filetime.saturating_sub(FILETIME_UNIX_DIFF);
    let secs = unix_100ns / 10_000_000;
    let nanos = ((unix_100ns % 10_000_000) * 100) as u32;
    SystemTime::UNIX_EPOCH + Duration::new(secs, nanos)
}

/// Combines the given [`SortKey`] and [`SortOrder`] into the corresponding
/// [`SortType`] used by [`everything_sdk`].
#[expect(clippy::enum_glob_use, reason = "Using glob imports improves readability in this match statement.")]
const fn convert_sort_type(key: SortKey, order: SortOrder) -> SortType {
    use SortKey::*;
    use SortType::*;
    use SortOrder::*;
    match (key, order) {
        (Name, Ascending) =>
            EVERYTHING_SORT_NAME_ASCENDING,
        (Name, Descending) =>
            EVERYTHING_SORT_NAME_DESCENDING,
        (TypeName, Ascending) =>
            EVERYTHING_SORT_TYPE_NAME_ASCENDING,
        (TypeName, Descending) =>
            EVERYTHING_SORT_TYPE_NAME_DESCENDING,
        (Path, Ascending) =>
            EVERYTHING_SORT_PATH_ASCENDING,
        (Path, Descending) =>
            EVERYTHING_SORT_PATH_DESCENDING,
        (Size, Ascending) =>
            EVERYTHING_SORT_SIZE_ASCENDING,
        (Size, Descending) =>
            EVERYTHING_SORT_SIZE_DESCENDING,
        (Extension, Ascending) =>
            EVERYTHING_SORT_EXTENSION_ASCENDING,
        (Extension, Descending) =>
            EVERYTHING_SORT_EXTENSION_DESCENDING,
        (DateCreated, Ascending) =>
            EVERYTHING_SORT_DATE_CREATED_ASCENDING,
        (DateCreated, Descending) =>
            EVERYTHING_SORT_DATE_CREATED_DESCENDING,
        (DateModified, Ascending) =>
            EVERYTHING_SORT_DATE_MODIFIED_ASCENDING,
        (DateModified, Descending) =>
            EVERYTHING_SORT_DATE_MODIFIED_DESCENDING,
        (DateAccessed, Ascending) =>
            EVERYTHING_SORT_DATE_ACCESSED_ASCENDING,
        (DateAccessed, Descending) =>
            EVERYTHING_SORT_DATE_ACCESSED_DESCENDING,
        (Attributes, Ascending) =>
            EVERYTHING_SORT_ATTRIBUTES_ASCENDING,
        (Attributes, Descending) =>
            EVERYTHING_SORT_ATTRIBUTES_DESCENDING,
    }
}
