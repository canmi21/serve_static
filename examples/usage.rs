/* examples/usage.rs */

use std::path::Path;

use serve_static::{listing, mime, path, range};

fn main() {
	let root = std::env::current_dir().expect("failed to get current directory");

	match path::resolve(&root, "/src/lib.rs", true) {
		Ok(fs_path) => println!("Resolved: {}", fs_path.display()),
		Err(e) => println!("Resolve error: {e}"),
	}

	if let Some(r) = range::parse("bytes=0-99", 1000) {
		println!("Range: start={}, length={}", r.start, r.length);
	}

	let content_type = mime::detect(Path::new("index.html"), &[]);
	println!("MIME: {content_type}");

	let tag = mime::etag(std::time::SystemTime::now(), 2048);
	println!("ETag: {tag}");

	let mut entries = vec![
		listing::Entry {
			name: "readme.txt".to_owned(),
			is_dir: false,
			size: Some(100),
			modified: None,
		},
		listing::Entry {
			name: "src".to_owned(),
			is_dir: true,
			size: None,
			modified: None,
		},
		listing::Entry {
			name: "Cargo.toml".to_owned(),
			is_dir: false,
			size: Some(500),
			modified: None,
		},
	];
	listing::sort(&mut entries);
	for entry in &entries {
		let kind = if entry.is_dir { "DIR " } else { "FILE" };
		println!("  {kind} {}", entry.name);
	}
}
