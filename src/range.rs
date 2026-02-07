/* src/range.rs */

use std::cmp;

/// A single byte range extracted from an HTTP Range header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
	/// Starting byte offset (zero-based).
	pub start: u64,
	/// Number of bytes in this range.
	pub length: u64,
}

/// Parses an HTTP Range header value (RFC 9110 section 14.1.2).
///
/// Supported formats:
/// - `bytes=100-199` single segment
/// - `bytes=100-` open-ended (from offset to EOF)
/// - `bytes=-200` suffix (last N bytes)
///
/// Returns `None` when the range is unsatisfiable (416) or malformed.
///
/// ```
/// let r = serve_static::range::parse("bytes=0-99", 1000);
/// assert_eq!(r, Some(serve_static::range::ByteRange { start: 0, length: 100 }));
/// ```
#[must_use]
pub fn parse(header: &str, total_size: u64) -> Option<ByteRange> {
	if total_size == 0 || !header.starts_with("bytes=") {
		return None;
	}

	let range_part = &header[6..];
	let (start_str, end_str) = range_part.split_once('-')?;

	if start_str.is_empty() {
		let suffix_len = end_str.parse::<u64>().ok()?;
		if suffix_len == 0 {
			return None;
		}
		let start = total_size.saturating_sub(suffix_len);
		return Some(ByteRange {
			start,
			length: total_size - start,
		});
	}

	let start = start_str.parse::<u64>().ok()?;
	let end = if end_str.is_empty() {
		total_size - 1
	} else {
		end_str.parse::<u64>().ok()?
	};

	if start > end || start >= total_size {
		return None;
	}

	let final_end = cmp::min(end, total_size - 1);

	Some(ByteRange {
		start,
		length: final_end - start + 1,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn single_segment() {
		let r = parse("bytes=100-199", 1000).unwrap();
		assert_eq!(
			r,
			ByteRange {
				start: 100,
				length: 100
			}
		);
	}

	#[test]
	fn open_ended() {
		let r = parse("bytes=100-", 1000).unwrap();
		assert_eq!(
			r,
			ByteRange {
				start: 100,
				length: 900
			}
		);
	}

	#[test]
	fn suffix() {
		let r = parse("bytes=-200", 1000).unwrap();
		assert_eq!(
			r,
			ByteRange {
				start: 800,
				length: 200
			}
		);
	}

	#[test]
	fn suffix_exceeds_size() {
		let r = parse("bytes=-5000", 1000).unwrap();
		assert_eq!(
			r,
			ByteRange {
				start: 0,
				length: 1000
			}
		);
	}

	#[test]
	fn start_beyond_size() {
		assert!(parse("bytes=1000-1100", 1000).is_none());
	}

	#[test]
	fn end_less_than_start() {
		assert!(parse("bytes=500-400", 1000).is_none());
	}

	#[test]
	fn non_bytes_unit() {
		assert!(parse("items=0-5", 1000).is_none());
	}

	#[test]
	fn malformed_values() {
		assert!(parse("bytes=abc-def", 1000).is_none());
	}

	#[test]
	fn end_truncated_to_size() {
		let r = parse("bytes=900-2000", 1000).unwrap();
		assert_eq!(
			r,
			ByteRange {
				start: 900,
				length: 100
			}
		);
	}

	#[test]
	fn suffix_zero() {
		assert!(parse("bytes=-0", 1000).is_none());
	}

	#[test]
	fn zero_total_size() {
		assert!(parse("bytes=0-0", 0).is_none());
	}
}
