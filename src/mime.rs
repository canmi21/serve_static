/* src/mime.rs */

use std::path::Path;
use std::time::SystemTime;

/// Detects the MIME type using a multi-strategy approach.
///
/// Detection order by priority:
/// 1. File extension guess (requires `extension` feature)
/// 2. Magic byte sniffing (requires `sniff` feature and non-empty `content`)
/// 3. UTF-8 heuristic: valid UTF-8 content yields `text/plain`
/// 4. Fallback: `application/octet-stream`
///
/// The `path` argument is only used for extension-based guessing and never
/// touches the filesystem. Pass `&[]` for `content` to skip byte-level checks.
///
/// ```
/// use std::path::Path;
/// let mime = serve_static::mime::detect(Path::new("data"), b"hello world");
/// assert_eq!(mime, "text/plain");
/// ```
#[must_use]
pub fn detect(path: &Path, content: &[u8]) -> String {
	#[cfg(feature = "extension")]
	if let Some(guess) = mime_guess::from_path(path).first()
		&& !(guess.type_() == "application" && guess.subtype() == "octet-stream")
	{
		return guess.to_string();
	}

	if !content.is_empty() {
		#[cfg(feature = "sniff")]
		if let Some(kind) = infer::get(content) {
			return kind.mime_type().to_owned();
		}

		if std::str::from_utf8(content).is_ok() {
			return "text/plain".to_owned();
		}
	}

	#[cfg(not(feature = "extension"))]
	let _ = path;

	"application/octet-stream".to_owned()
}

/// Generates a weak ETag from file metadata.
///
/// Format: `W/"<mtime_hex_nanos>-<size_hex>"`
///
/// ```
/// use std::time::{SystemTime, Duration, UNIX_EPOCH};
/// let t = UNIX_EPOCH + Duration::from_secs(100);
/// let tag = serve_static::mime::etag(t, 500);
/// assert!(tag.starts_with("W/\""));
/// assert!(tag.contains("1f4"));
/// ```
#[must_use]
pub fn etag(modified: SystemTime, size: u64) -> String {
	let duration = modified
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default();
	let nanos = duration.as_nanos();
	format!("W/\"{nanos:x}-{size:x}\"")
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::{Duration, UNIX_EPOCH};

	#[test]
	#[cfg(feature = "extension")]
	fn html_by_extension() {
		assert_eq!(detect(Path::new("index.html"), &[]), "text/html");
	}

	#[test]
	#[cfg(feature = "extension")]
	fn json_by_extension() {
		assert_eq!(detect(Path::new("data.json"), &[]), "application/json");
	}

	#[test]
	#[cfg(feature = "extension")]
	fn jpg_case_insensitive() {
		assert_eq!(detect(Path::new("PHOTO.JPG"), &[]), "image/jpeg");
	}

	#[test]
	#[cfg(feature = "sniff")]
	fn png_by_magic_bytes() {
		let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
		assert_eq!(detect(Path::new("unknown"), &png_header), "image/png");
	}

	#[test]
	#[cfg(all(feature = "extension", feature = "sniff"))]
	fn extension_overrides_sniff() {
		let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
		// .css extension must win over PNG magic bytes
		assert_eq!(detect(Path::new("style.css"), &png_header), "text/css");
	}

	#[test]
	fn utf8_heuristic() {
		assert_eq!(
			detect(Path::new("README"), b"Just some plain text"),
			"text/plain"
		);
	}

	#[test]
	fn binary_fallback() {
		assert_eq!(
			detect(Path::new("blob"), &[0x00, 0x01, 0x02, 0xFF, 0xFE]),
			"application/octet-stream"
		);
	}

	#[test]
	fn no_ext_no_content() {
		assert_eq!(detect(Path::new("noext"), &[]), "application/octet-stream");
	}

	#[test]
	fn etag_format() {
		let t = UNIX_EPOCH + Duration::from_secs(100);
		let tag = etag(t, 500);
		assert!(tag.starts_with("W/\""));
		assert!(tag.ends_with('"'));
		assert!(tag.contains("1f4"));
	}

	#[test]
	fn etag_zero_epoch() {
		let tag = etag(UNIX_EPOCH, 0);
		assert_eq!(tag, "W/\"0-0\"");
	}

	#[test]
	fn etag_pre_epoch() {
		let t = UNIX_EPOCH - Duration::from_secs(100);
		// duration_since(UNIX_EPOCH) fails → unwrap_or_default → nanos=0
		let tag = etag(t, 42);
		assert_eq!(tag, "W/\"0-2a\"");
	}
}
