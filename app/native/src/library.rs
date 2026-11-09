//! `Library`: the open documents, kept in Rust.
//!
//! TSX holds a document's numeric id; the source text, its sections and
//! outline stay here, so a 50 MB document is never copied into the
//! interface's JavaScript heap: rows ask for their own section's Markdown
//! when the virtualized list shows them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use nashir_engine::{highlight_markdown, load_path, search_document, Document, LoadOptions};
use nsh_core::prelude::*;
use serde::Serialize;

use crate::util::{background, base64, image_mime, path_from_uri};

struct Open {
    path: PathBuf,
    doc: Arc<Document>,
    modified_ms: u64,
}

fn store() -> &'static Mutex<HashMap<u32, Open>> {
    static STORE: OnceLock<Mutex<HashMap<u32, Open>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn with_doc<T>(id: u32, f: impl FnOnce(&Open) -> T) -> Option<T> {
    store().lock().ok()?.get(&id).map(f)
}

/// What `open` reports about a document (JSON for TSX).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta<'a> {
    id: u32,
    path: String,
    name: String,
    folder: String,
    bytes: u64,
    encoding: &'a str,
    lossy: bool,
    modified: u64,
    direction: &'a str,
    /// One character per section: `r` (right-to-left) or `l`.
    section_directions: String,
    outline: &'a [nashir_engine::OutlineItem],
    stats: nashir_engine::Stats,
    front_matter: bool,
}

/// The documents Nashir has open.
#[module]
pub struct Library;

#[module]
impl Library {
    /// Opens the Markdown file at `path` (a path or a `file:` URL), reading
    /// and parsing it on a background thread. Resolves with the document's
    /// metadata as JSON (`DocumentMeta` in `ui/native.ts`); rejects with an
    /// error code: `not_found`, `not_a_file`, `permission_denied`,
    /// `too_large:<bytes>:<max>`, `binary` or `io:<message>`.
    pub async fn open(path: String, max_megabytes: f64) -> Result<String, String> {
        let path = path_from_uri(&path);
        let max = (max_megabytes.clamp(1.0, 512.0) * 1024.0 * 1024.0) as u64;
        let work_path = path.clone();
        let parsed = background(
            move || -> Result<(Document, u64, u64, &'static str, bool), String> {
                let loaded =
                    load_path(&work_path, LoadOptions { max_bytes: max }).map_err(|e| e.to_string())?;
                let (bytes, modified, encoding, lossy) =
                    (loaded.bytes, loaded.modified_ms, loaded.encoding, loaded.lossy);
                Ok((Document::parse(loaded.text), bytes, modified, encoding, lossy))
            },
        )
        .await??;
        let (doc, bytes, modified, encoding, lossy) = parsed;
        let canonical = std::fs::canonicalize(&path).unwrap_or(path.clone());
        register(doc, strip_verbatim(&canonical), bytes, modified, encoding, lossy)
    }

    /// Opens Nashir's built-in welcome document in `lang` (`ar`, `en` or
    /// `fr`): a tour of what the reader shows, Arabic, tables, code and
    /// Mermaid diagrams included. Resolves with its metadata as JSON.
    pub async fn open_welcome(lang: String) -> Result<String, String> {
        let text = match lang.as_str() {
            "en" => include_str!("../welcome/en.md"),
            "fr" => include_str!("../welcome/fr.md"),
            _ => include_str!("../welcome/ar.md"),
        };
        let name = match lang.as_str() {
            "en" => "Welcome.md",
            "fr" => "Bienvenue.md",
            _ => "مرحبا.md",
        };
        let doc = background(move || Document::parse(text.to_owned())).await?;
        register(doc, name.to_owned(), text.len() as u64, 0, "utf-8", false)
    }

    /// The Markdown of section `index` of document `id`, with the footnote
    /// and link definitions it needs; empty when either is unknown.
    pub fn section(id: u32, index: u32) -> String {
        with_doc(id, |o| o.doc.section_markdown(index as usize)).unwrap_or_default()
    }

    /// Like `section`, with every match of `query` wrapped in `<mark>`.
    pub fn highlighted(id: u32, index: u32, query: String) -> String {
        with_doc(id, |o| {
            let source = o.doc.section_source(index as usize);
            let (marked, _) = highlight_markdown(&source, &query);
            o.doc.complete(marked)
        })
        .unwrap_or_default()
    }

    /// Searches document `id` for `query` (case-insensitive, Arabic-aware)
    /// on a background thread. Resolves with `SearchResult` JSON.
    pub async fn search(id: u32, query: String, limit: u32) -> Result<String, String> {
        let doc = with_doc(id, |o| o.doc.clone()).ok_or_else(|| "not_found".to_owned())?;
        background(move || {
            serde_json::to_string(&search_document(&doc, &query, limit.max(1) as usize)).unwrap_or_default()
        })
        .await
    }

    /// The section holding the heading with `anchor` (`#install`), or -1.
    pub fn anchor(id: u32, anchor: String) -> i32 {
        with_doc(id, |o| o.doc.section_for_anchor(&anchor).map(|s| s as i32))
            .flatten()
            .unwrap_or(-1)
    }

    /// The outline entry current at section `index`, or -1.
    pub fn heading_at(id: u32, index: u32) -> i32 {
        with_doc(id, |o| {
            o.doc.heading_for_section(index as usize).map(|h| h as i32)
        })
        .flatten()
        .unwrap_or(-1)
    }

    /// Whether the file of document `id` changed on disk since it was opened
    /// (its modification time moved, or it was removed).
    pub fn changed(id: u32) -> bool {
        with_doc(id, |o| {
            // Built-in documents (modified_ms 0) have no file to watch.
            o.modified_ms != 0
                && match nashir_engine::loader::modified_ms(&o.path) {
                    Some(m) => m != o.modified_ms,
                    None => true,
                }
        })
        .unwrap_or(false)
    }

    /// Whether `path` still exists as a file.
    pub fn exists(path: String) -> bool {
        path_from_uri(&path).is_file()
    }

    /// Forgets document `id`, freeing its memory.
    pub fn close(id: u32) {
        if let Ok(mut s) = store().lock() {
            s.remove(&id);
        }
    }

    /// A local image referenced by document `id` (`src` relative to the
    /// document's folder, an absolute path or a `file:` URL) as a `data:`
    /// URI, or an empty string when it cannot be read. Images over 20 MB are
    /// refused.
    pub async fn image(id: u32, src: String) -> Result<String, String> {
        let base = with_doc(id, |o| o.path.parent().map(Path::to_path_buf))
            .flatten()
            .ok_or_else(|| "not_found".to_owned())?;
        background(move || {
            let clean = src.split(['?', '#']).next().unwrap_or("").to_owned();
            let candidate = if clean.starts_with("file:") {
                path_from_uri(&clean)
            } else {
                PathBuf::from(&clean)
            };
            let path = if candidate.is_absolute() {
                candidate
            } else {
                base.join(percent(&clean))
            };
            let Some(mime) = image_mime(&path) else {
                return String::new();
            };
            match std::fs::metadata(&path) {
                Ok(m) if m.is_file() && m.len() <= 20 * 1024 * 1024 => {}
                _ => return String::new(),
            }
            match std::fs::read(&path) {
                Ok(bytes) => format!("data:{mime};base64,{}", base64(&bytes)),
                Err(_) => String::new(),
            }
        })
        .await
    }

    /// The number of open documents (for diagnostics and tests).
    pub fn open_count() -> u32 {
        store().lock().map(|s| s.len() as u32).unwrap_or(0)
    }
}

/// Stores a parsed document and describes it to the interface.
fn register(
    doc: Document,
    display_path: String,
    bytes: u64,
    modified: u64,
    encoding: &str,
    lossy: bool,
) -> Result<String, String> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = Path::new(&display_path);
    let meta = Meta {
        id,
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        folder: path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: display_path.clone(),
        bytes,
        encoding,
        lossy,
        modified,
        direction: doc.direction.as_str(),
        section_directions: doc
            .sections
            .iter()
            .map(|s| {
                if s.direction == nashir_engine::Direction::Rtl {
                    'r'
                } else {
                    'l'
                }
            })
            .collect(),
        outline: &doc.outline,
        stats: doc.stats,
        front_matter: doc.has_front_matter(),
    };
    let json = serde_json::to_string(&meta).map_err(|e| format!("io:{e}"))?;
    store().lock().map_err(|_| "io:lock".to_owned())?.insert(
        id,
        Open {
            path: PathBuf::from(display_path),
            doc: Arc::new(doc),
            modified_ms: modified,
        },
    );
    Ok(json)
}

fn percent(s: &str) -> String {
    // Relative Markdown links often encode spaces (`my%20image.png`).
    let mut out = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `\\?\C:\x` (what `canonicalize` gives on Windows) → `C:\x`.
fn strip_verbatim(p: &Path) -> String {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_owned()
    } else {
        s.into_owned()
    }
}
