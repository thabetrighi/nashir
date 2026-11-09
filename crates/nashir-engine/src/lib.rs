//! # nashir-engine
//!
//! The platform-independent core of **Nashir (ناشر)**, a native Markdown
//! reader built on Native Shell (NSH).
//!
//! Nothing in this crate depends on NSH: it is plain Rust, tested with
//! `cargo test`, and called from the app's native module (`app/native`),
//! which exposes it to the TSX interface.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`loader`] | Reading files safely: size limits, binary detection, encodings, BOM |
//! | [`document`] | Splitting a document into renderable sections, outline, stats |
//! | [`direction`] | Detecting the base direction (RTL / LTR) of text |
//! | [`text`] | Arabic-aware normalisation used by search |
//! | [`search`] | Finding matches with snippets, and highlighting them |
//! | [`mermaid`] | Parsing and laying out Mermaid diagrams into a native scene |

// Drawing helpers take a box (x, y, w, h) plus style values; grouping them
// in structs would only add noise at every call site.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod direction;
pub mod document;
pub mod loader;
pub mod mermaid;
pub mod search;
pub mod slug;
pub mod text;

pub use direction::{detect_direction, Direction};
pub use document::{Document, OutlineItem, Section, Stats};
pub use loader::{load_bytes, load_path, LoadError, LoadOptions, Loaded};
pub use search::{highlight_markdown, search_document, SearchHit, SearchResult};

/// The file extensions Nashir opens as Markdown.
pub const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "mkd", "mkdn", "mdwn", "mdtxt", "txt"];

/// Whether `path` has one of [`MARKDOWN_EXTENSIONS`] (case-insensitive).
pub fn is_markdown_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    match lower.rsplit_once('.') {
        Some((_, ext)) => MARKDOWN_EXTENSIONS.contains(&ext),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_extensions() {
        assert!(is_markdown_path("C:\\docs\\README.md"));
        assert!(is_markdown_path("/a/b/notes.MARKDOWN"));
        assert!(is_markdown_path("x.mdown"));
        assert!(is_markdown_path("x.mkd"));
        assert!(!is_markdown_path("x.pdf"));
        assert!(!is_markdown_path("README"));
    }
}
