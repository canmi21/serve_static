/* src/listing.rs */

use std::time::SystemTime;

/// A single directory entry for use in directory listings.
///
/// Callers populate these from their own I/O layer, then pass them
/// to [`sort`] for canonical ordering.
///
/// ```
/// let entry = serve_static::listing::Entry {
///     name: "readme.txt".to_owned(),
///     is_dir: false,
///     size: Some(1024),
///     modified: None,
/// };
/// assert!(!entry.is_dir);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
	/// File or directory name without any path prefix.
	pub name: String,
	/// Whether this entry is a directory.
	pub is_dir: bool,
	/// File size in bytes. `None` for directories.
	pub size: Option<u64>,
	/// Last modification time.
	pub modified: Option<SystemTime>,
}

/// Sorts entries with directories first, then alphabetically (case-insensitive).
///
/// ```
/// use serve_static::listing::{Entry, sort};
///
/// let mut entries = vec![
///     Entry { name: "z.txt".to_owned(), is_dir: false, size: Some(10), modified: None },
///     Entry { name: "a_dir".to_owned(), is_dir: true, size: None, modified: None },
/// ];
/// sort(&mut entries);
/// assert!(entries[0].is_dir);
/// assert_eq!(entries[0].name, "a_dir");
/// ```
pub fn sort(entries: &mut [Entry]) {
	entries.sort_by_cached_key(|entry| (!entry.is_dir, entry.name.to_lowercase()));
}

#[cfg(test)]
mod tests {
	use super::*;

	fn file(name: &str) -> Entry {
		Entry {
			name: name.to_owned(),
			is_dir: false,
			size: Some(100),
			modified: None,
		}
	}

	fn dir(name: &str) -> Entry {
		Entry {
			name: name.to_owned(),
			is_dir: true,
			size: None,
			modified: None,
		}
	}

	#[test]
	fn directories_before_files() {
		let mut entries = vec![file("b.txt"), dir("docs"), file("a.txt"), dir("assets")];
		sort(&mut entries);
		assert!(entries[0].is_dir);
		assert!(entries[1].is_dir);
		assert!(!entries[2].is_dir);
		assert!(!entries[3].is_dir);
	}

	#[test]
	fn case_insensitive_ordering() {
		let mut entries = vec![file("Banana"), file("apple"), file("Cherry")];
		sort(&mut entries);
		assert_eq!(entries[0].name, "apple");
		assert_eq!(entries[1].name, "Banana");
		assert_eq!(entries[2].name, "Cherry");
	}

	#[test]
	fn empty_list() {
		let mut entries: Vec<Entry> = vec![];
		sort(&mut entries);
		assert!(entries.is_empty());
	}

	#[test]
	fn single_entry() {
		let mut entries = vec![file("only.txt")];
		sort(&mut entries);
		assert_eq!(entries.len(), 1);
	}

	#[test]
	fn mixed_sort() {
		let mut entries = vec![
			file("readme.md"),
			dir("src"),
			file("Cargo.toml"),
			dir("examples"),
		];
		sort(&mut entries);
		assert_eq!(entries[0].name, "examples");
		assert_eq!(entries[1].name, "src");
		assert_eq!(entries[2].name, "Cargo.toml");
		assert_eq!(entries[3].name, "readme.md");
	}
}
