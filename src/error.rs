/* src/error.rs */

use std::path::PathBuf;

use thiserror::Error;

/// All errors that serve_static can produce.
#[derive(Debug, Error)]
pub enum Error {
	/// The root path is invalid or does not exist.
	#[error("invalid root path '{path}': {source}")]
	InvalidRoot {
		/// The path that failed to canonicalize.
		path: PathBuf,
		/// The underlying I/O error.
		source: std::io::Error,
	},

	/// The URI contains invalid UTF-8 percent encoding.
	#[error("invalid URI encoding: {0}")]
	InvalidEncoding(#[from] std::str::Utf8Error),

	/// The decoded URI contains a null byte.
	#[error("null byte in URI path")]
	NullByte,

	/// A symlink resolved to a path outside the root directory.
	#[error("path traversal detected via symlink")]
	SymlinkTraversal,

	/// An I/O error during path resolution that is not NotFound.
	#[error("path resolution security error: {0}")]
	SecurityIo(std::io::Error),
}
