# Serve Static

Headless utilities for static file serving: path jail, range parsing, MIME detection, directory listing.

`serve_static` provides framework-agnostic building blocks for serving static files. It performs no I/O and depends on no async runtime, so it integrates with any HTTP server (axum, actix-web, warp, hyper, or your own).

## Features

- **Path Resolution**: Safely resolve URI paths to filesystem paths with directory traversal protection, percent-decoding, and optional symlink detection.
- **Range Parsing**: RFC 9110 compliant HTTP Range header parsing for single byte ranges.
- **MIME Detection**: Multi-strategy content type detection via file extension, magic bytes, and UTF-8 heuristic.
- **ETag Generation**: Weak ETag generation from file metadata (mtime + size).
- **Directory Listing**: Structured data model and sorting for directory entries (directories first, case-insensitive alphabetical).

## Usage Examples

Check the `examples` directory for runnable code:

- **Full Demo**: [`examples/usage.rs`](examples/usage.rs) - Demonstrates all modules working together.

## Installation

```toml
[dependencies]
serve_static = { version = "0.1", features = ["full"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `sniff` | Enables magic-byte MIME sniffing via `infer` - enabled by default. |
| `extension` | Enables file extension MIME guessing via `mime_guess` - enabled by default. |
| `full` | Enables all features above. |

## License

Released under the MIT License © 2026 [Canmi](https://github.com/canmi21)
