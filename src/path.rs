/* src/path.rs */

use std::path::{Component, Path, PathBuf};

use crate::error::Error;

/// Resolves a URI path into a physical filesystem path under `root`.
///
/// Provides protection against directory traversal attacks by:
/// - Percent-decoding the URI (`%2e%2e` becomes `..`)
/// - Normalizing path components in memory (no filesystem access)
/// - Preventing `..` from escaping the root boundary
/// - Optionally detecting symlink-based traversal via `canonicalize()`
///
/// When the target file does not exist but the path is syntactically safe,
/// `Ok(path)` is still returned. The caller handles 404 logic.
///
/// ```
/// let root = std::env::temp_dir();
/// let result = serve_static::path::resolve(&root, "/file.txt", true);
/// assert!(result.is_ok());
/// ```
pub fn resolve(root: &Path, uri: &str, allow_symlinks: bool) -> Result<PathBuf, Error> {
	let root = root.canonicalize().map_err(|source| Error::InvalidRoot {
		path: root.to_path_buf(),
		source,
	})?;

	let decoded = percent_encoding::percent_decode_str(uri).decode_utf8()?;
	if decoded.contains('\0') {
		return Err(Error::NullByte);
	}
	let mut resolved = root.clone();

	for component in Path::new(decoded.as_ref()).components() {
		match component {
			Component::Normal(c) => resolved.push(c),
			Component::ParentDir => {
				if resolved != root {
					resolved.pop();
				}
			}
			Component::RootDir | Component::CurDir | Component::Prefix(_) => {}
		}
	}

	if !allow_symlinks {
		match resolved.canonicalize() {
			Ok(canonical) => {
				if !canonical.starts_with(&root) {
					return Err(Error::SymlinkTraversal);
				}
				return Ok(canonical);
			}
			Err(e) => {
				if e.kind() == std::io::ErrorKind::NotFound {
					// The final target does not exist, but intermediate
					// symlinks could still escape root. Walk up to the
					// nearest existing ancestor and verify it stays
					// inside root.
					let mut ancestor = resolved.clone();
					while ancestor.pop() {
						if ancestor == root {
							break;
						}
						match ancestor.canonicalize() {
							Ok(canonical) => {
								if !canonical.starts_with(&root) {
									return Err(Error::SymlinkTraversal);
								}
								break;
							}
							Err(inner)
								if inner.kind() == std::io::ErrorKind::NotFound => {}

							Err(inner) => return Err(Error::SecurityIo(inner)),
						}
					}
					return Ok(resolved);
				}
				return Err(Error::SecurityIo(e));
			}
		}
	}

	Ok(resolved)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_root() -> tempfile::TempDir {
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(dir.path().join("assets/images")).unwrap();
		std::fs::write(dir.path().join("assets/images/logo.png"), b"png").unwrap();
		std::fs::write(dir.path().join("index.html"), b"<html>").unwrap();
		dir
	}

	#[test]
	fn normal_path() {
		let root = make_root();
		let result = resolve(root.path(), "/assets/images/logo.png", false).unwrap();
		assert!(result.ends_with("assets/images/logo.png"));
	}

	#[test]
	fn traversal_attack() {
		let root = make_root();
		let result = resolve(root.path(), "/../../etc/passwd", true).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert!(result.starts_with(&canon_root));
	}

	#[test]
	fn percent_encoded_traversal() {
		let root = make_root();
		let result = resolve(root.path(), "/%2e%2e/%2e%2e/etc/shadow", true).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert!(result.starts_with(&canon_root));
	}

	#[test]
	fn redundant_components() {
		let root = make_root();
		let result = resolve(root.path(), "/assets//images/./logo.png", false).unwrap();
		assert!(result.ends_with("assets/images/logo.png"));
	}

	#[test]
	fn missing_file_returns_ok() {
		let root = make_root();
		let result = resolve(root.path(), "/missing.html", false).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert_eq!(result, canon_root.join("missing.html"));
	}

	#[test]
	fn absolute_uri_treated_as_relative() {
		let root = make_root();
		let result = resolve(root.path(), "/etc/passwd", true).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert_eq!(result, canon_root.join("etc/passwd"));
	}

	#[cfg(unix)]
	#[test]
	fn symlink_traversal_blocked() {
		let root = make_root();
		let outside = tempfile::tempdir().unwrap();
		let secret = outside.path().join("secret.txt");
		std::fs::write(&secret, b"secret").unwrap();

		let link = root.path().join("link.txt");
		std::os::unix::fs::symlink(&secret, &link).unwrap();

		let result = resolve(root.path(), "/link.txt", false);
		assert!(matches!(result, Err(Error::SymlinkTraversal)));
	}

	#[cfg(unix)]
	#[test]
	fn symlink_dir_nonexistent_target_blocked() {
		let root = make_root();
		let outside = tempfile::tempdir().unwrap();

		// Symlink an entire directory to an outside location.
		let link = root.path().join("evil");
		std::os::unix::fs::symlink(outside.path(), &link).unwrap();

		// Requesting a non-existent file *through* the symlink must
		// still be caught, even though canonicalize fails with NotFound.
		let result = resolve(root.path(), "/evil/nonexistent.txt", false);
		assert!(matches!(result, Err(Error::SymlinkTraversal)));
	}

	#[cfg(unix)]
	#[test]
	fn symlink_allowed_when_flag_set() {
		let root = make_root();
		let outside = tempfile::tempdir().unwrap();
		let secret = outside.path().join("secret.txt");
		std::fs::write(&secret, b"secret").unwrap();

		let link = root.path().join("link.txt");
		std::os::unix::fs::symlink(&secret, &link).unwrap();

		let result = resolve(root.path(), "/link.txt", true).unwrap();
		assert!(result.ends_with("link.txt"));
	}

	#[test]
	fn invalid_root() {
		let result = resolve(Path::new("/nonexistent_root_dir_xyz"), "/file", false);
		assert!(matches!(result, Err(Error::InvalidRoot { .. })));
	}

	#[test]
	fn empty_uri() {
		let root = make_root();
		let result = resolve(root.path(), "", true).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert_eq!(result, canon_root);
	}

	#[test]
	fn root_uri() {
		let root = make_root();
		let result = resolve(root.path(), "/", true).unwrap();
		let canon_root = root.path().canonicalize().unwrap();
		assert_eq!(result, canon_root);
	}

	// ── happy-path: solidify interface behaviour ──

	#[test]
	fn double_encoding_stays_literal() {
		// %252e%252e decodes once to "%2e%2e", which is a normal filename,
		// NOT a traversal component.
		let root = make_root();
		let result = resolve(root.path(), "/%252e%252e", true).unwrap();
		assert!(result.ends_with("%2e%2e"));
	}

	#[test]
	fn unicode_filename() {
		let root = make_root();
		let result = resolve(root.path(), "/%E4%B8%AD%E6%96%87.txt", true).unwrap();
		assert!(result.ends_with("中文.txt"));
	}

	#[test]
	fn traversal_clamped_then_descend() {
		// Many ../ followed by a valid descent must resolve inside root.
		let root = make_root();
		let result = resolve(root.path(), "/../../assets/images/logo.png", false).unwrap();
		assert!(result.ends_with("assets/images/logo.png"));
	}

	#[test]
	fn query_string_becomes_filename() {
		// The library does NOT strip query strings; callers must do it.
		let root = make_root();
		let result = resolve(root.path(), "/file.txt?key=value", true).unwrap();
		assert!(result.ends_with("file.txt?key=value"));
	}

	#[test]
	fn fragment_becomes_filename() {
		// Same as query strings — fragments are the caller's concern.
		let root = make_root();
		let result = resolve(root.path(), "/file.txt%23section", true).unwrap();
		assert!(result.ends_with("file.txt#section"));
	}

	#[test]
	fn deeply_nested_path() {
		let root = make_root();
		let result = resolve(root.path(), "/a/b/c/d/e/f/g/h", true).unwrap();
		assert!(result.ends_with("a/b/c/d/e/f/g/h"));
	}

	#[test]
	fn pure_traversal_resolves_to_root() {
		let root = make_root();
		let canon_root = root.path().canonicalize().unwrap();
		let result = resolve(root.path(), "/../../../../..", true).unwrap();
		assert_eq!(result, canon_root);
	}

	// ── error-path: invalid inputs must return Err ──

	#[test]
	fn null_byte_rejected() {
		let root = make_root();
		let result = resolve(root.path(), "/file%00.txt", true);
		assert!(matches!(result, Err(Error::NullByte)));
	}

	#[test]
	fn null_byte_rejected_no_symlinks() {
		let root = make_root();
		let result = resolve(root.path(), "/%00", false);
		assert!(matches!(result, Err(Error::NullByte)));
	}

	#[test]
	fn invalid_utf8_encoding_rejected() {
		// %C3%28 is invalid UTF-8 (0xC3 expects a continuation byte, 0x28 is not).
		let root = make_root();
		let result = resolve(root.path(), "/%C3%28", true);
		assert!(matches!(result, Err(Error::InvalidEncoding(_))));
	}
}
