//! Reading documents from disk safely.
//!
//! Every file is checked before it is read: it must exist, be a regular
//! file, and stay under [`LoadOptions::max_bytes`]. Its bytes are then
//! checked for binary content and decoded: UTF-8 (with or without a BOM),
//! UTF-16 LE/BE with a BOM, and — as a last resort — Windows-1256 (Arabic
//! Windows), which many older Arabic text files use.

use std::fmt;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Limits applied when loading.
#[derive(Debug, Clone, Copy)]
pub struct LoadOptions {
    /// The largest file accepted, in bytes. Default: 64 MiB.
    pub max_bytes: u64,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
        }
    }
}

/// A file read and decoded.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub text: String,
    /// The encoding the bytes were decoded from: `utf-8`, `utf-8-bom`,
    /// `utf-16le`, `utf-16be` or `windows-1256`.
    pub encoding: &'static str,
    pub bytes: u64,
    /// Last modification, milliseconds since the Unix epoch (0 if unknown).
    pub modified_ms: u64,
    /// Whether invalid sequences were replaced while decoding.
    pub lossy: bool,
}

/// Why a file could not be loaded. Each variant has a stable `code()` the
/// interface translates into a message in the user's language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    NotFound,
    NotAFile,
    PermissionDenied,
    TooLarge { bytes: u64, max: u64 },
    Binary,
    Io(String),
}

impl LoadError {
    /// A stable, language-independent code (`not_found`, `too_large`, …).
    pub fn code(&self) -> &'static str {
        match self {
            LoadError::NotFound => "not_found",
            LoadError::NotAFile => "not_a_file",
            LoadError::PermissionDenied => "permission_denied",
            LoadError::TooLarge { .. } => "too_large",
            LoadError::Binary => "binary",
            LoadError::Io(_) => "io",
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::NotFound => write!(f, "not_found"),
            LoadError::NotAFile => write!(f, "not_a_file"),
            LoadError::PermissionDenied => write!(f, "permission_denied"),
            LoadError::TooLarge { bytes, max } => write!(f, "too_large:{bytes}:{max}"),
            LoadError::Binary => write!(f, "binary"),
            LoadError::Io(e) => write!(f, "io:{e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Reads and decodes the file at `path`.
pub fn load_path(path: &Path, options: LoadOptions) -> Result<Loaded, LoadError> {
    let meta = fs::metadata(path).map_err(map_io)?;
    if !meta.is_file() {
        return Err(LoadError::NotAFile);
    }
    if meta.len() > options.max_bytes {
        return Err(LoadError::TooLarge {
            bytes: meta.len(),
            max: options.max_bytes,
        });
    }
    let modified_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let bytes = fs::read(path).map_err(map_io)?;
    // The file may have grown between `metadata` and `read`.
    if bytes.len() as u64 > options.max_bytes {
        return Err(LoadError::TooLarge {
            bytes: bytes.len() as u64,
            max: options.max_bytes,
        });
    }
    let mut loaded = load_bytes(&bytes)?;
    loaded.modified_ms = modified_ms;
    Ok(loaded)
}

/// The modification time of `path` in milliseconds, for change detection.
pub fn modified_ms(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

fn map_io(e: std::io::Error) -> LoadError {
    match e.kind() {
        std::io::ErrorKind::NotFound => LoadError::NotFound,
        std::io::ErrorKind::PermissionDenied => LoadError::PermissionDenied,
        _ => LoadError::Io(e.to_string()),
    }
}

/// Decodes `bytes` as a text document.
pub fn load_bytes(bytes: &[u8]) -> Result<Loaded, LoadError> {
    let len = bytes.len() as u64;
    // UTF-16 with a BOM.
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok(decode_utf16(&bytes[2..], true, len));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok(decode_utf16(&bytes[2..], false, len));
    }
    let (body, bom) = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        Some(rest) => (rest, true),
        None => (bytes, false),
    };
    if looks_binary(body) {
        return Err(LoadError::Binary);
    }
    match std::str::from_utf8(body) {
        Ok(text) => Ok(Loaded {
            text: normalize_newlines(text),
            encoding: if bom { "utf-8-bom" } else { "utf-8" },
            bytes: len,
            modified_ms: 0,
            lossy: false,
        }),
        Err(_) => {
            // Not UTF-8. If most invalid bytes are in the Arabic range of
            // Windows-1256, decode as such; otherwise replace them.
            if likely_windows_1256(body) {
                Ok(Loaded {
                    text: normalize_newlines(&decode_windows_1256(body)),
                    encoding: "windows-1256",
                    bytes: len,
                    modified_ms: 0,
                    lossy: false,
                })
            } else {
                Ok(Loaded {
                    text: normalize_newlines(&String::from_utf8_lossy(body)),
                    encoding: "utf-8",
                    bytes: len,
                    modified_ms: 0,
                    lossy: true,
                })
            }
        }
    }
}

fn decode_utf16(body: &[u8], le: bool, len: u64) -> Loaded {
    let units: Vec<u16> = body
        .chunks_exact(2)
        .map(|c| {
            if le {
                u16::from_le_bytes([c[0], c[1]])
            } else {
                u16::from_be_bytes([c[0], c[1]])
            }
        })
        .collect();
    let text = String::from_utf16_lossy(&units);
    let lossy = text.contains('\u{FFFD}');
    Loaded {
        text: normalize_newlines(&text),
        encoding: if le { "utf-16le" } else { "utf-16be" },
        bytes: len,
        modified_ms: 0,
        lossy,
    }
}

/// A NUL byte in the first 8 KiB, or a high share of control bytes,
/// marks a binary file (an image, an archive, a PDF…).
fn looks_binary(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(8192)];
    if head.contains(&0) {
        return true;
    }
    if head.is_empty() {
        return false;
    }
    let control = head
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20 && b != 0x1B))
        .count();
    control * 10 > head.len()
}

fn normalize_newlines(text: &str) -> String {
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_owned()
    }
}

fn likely_windows_1256(bytes: &[u8]) -> bool {
    let high: Vec<u8> = bytes.iter().copied().filter(|&b| b >= 0x80).take(4096).collect();
    if high.is_empty() {
        return false;
    }
    // Arabic letters live at 0xC1..=0xFE (with a few Latin holes).
    let arabic = high.iter().filter(|&&b| (0xC1..=0xFE).contains(&b)).count();
    arabic * 10 >= high.len() * 6
}

/// Windows-1256 → Unicode, bytes 0x80..=0xFF.
const CP1256_HIGH: [char; 128] = [
    '\u{20AC}', '\u{067E}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0679}', '\u{2039}', '\u{0152}', '\u{0686}', '\u{0698}', '\u{0688}',
    '\u{06AF}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{06A9}', '\u{2122}', '\u{0691}', '\u{203A}', '\u{0153}', '\u{200C}', '\u{200D}', '\u{06BA}',
    '\u{00A0}', '\u{060C}', '\u{00A2}', '\u{00A3}', '\u{00A4}', '\u{00A5}', '\u{00A6}', '\u{00A7}',
    '\u{00A8}', '\u{00A9}', '\u{06BE}', '\u{00AB}', '\u{00AC}', '\u{00AD}', '\u{00AE}', '\u{00AF}',
    '\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00B3}', '\u{00B4}', '\u{00B5}', '\u{00B6}', '\u{00B7}',
    '\u{00B8}', '\u{00B9}', '\u{061B}', '\u{00BB}', '\u{00BC}', '\u{00BD}', '\u{00BE}', '\u{061F}',
    '\u{06C1}', '\u{0621}', '\u{0622}', '\u{0623}', '\u{0624}', '\u{0625}', '\u{0626}', '\u{0627}',
    '\u{0628}', '\u{0629}', '\u{062A}', '\u{062B}', '\u{062C}', '\u{062D}', '\u{062E}', '\u{062F}',
    '\u{0630}', '\u{0631}', '\u{0632}', '\u{0633}', '\u{0634}', '\u{0635}', '\u{0636}', '\u{00D7}',
    '\u{0637}', '\u{0638}', '\u{0639}', '\u{063A}', '\u{0640}', '\u{0641}', '\u{0642}', '\u{0643}',
    '\u{00E0}', '\u{0644}', '\u{00E2}', '\u{0645}', '\u{0646}', '\u{0647}', '\u{0648}', '\u{00E7}',
    '\u{00E8}', '\u{00E9}', '\u{00EA}', '\u{00EB}', '\u{0649}', '\u{064A}', '\u{00EE}', '\u{00EF}',
    '\u{064B}', '\u{064C}', '\u{064D}', '\u{064E}', '\u{00F4}', '\u{064F}', '\u{0650}', '\u{00F7}',
    '\u{0651}', '\u{00F9}', '\u{0652}', '\u{00FB}', '\u{00FC}', '\u{200E}', '\u{200F}', '\u{06D2}',
];

fn decode_windows_1256(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b < 0x80 {
                b as char
            } else {
                CP1256_HIGH[(b - 0x80) as usize]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_with_and_without_bom() {
        let l = load_bytes("# مرحبا\r\nworld".as_bytes()).unwrap();
        assert_eq!(l.text, "# مرحبا\nworld");
        assert_eq!(l.encoding, "utf-8");
        let mut b = vec![0xEF, 0xBB, 0xBF];
        b.extend_from_slice("نص".as_bytes());
        let l = load_bytes(&b).unwrap();
        assert_eq!(l.text, "نص");
        assert_eq!(l.encoding, "utf-8-bom");
    }

    #[test]
    fn utf16() {
        let mut b = vec![0xFF, 0xFE];
        for u in "سلام".encode_utf16() {
            b.extend_from_slice(&u.to_le_bytes());
        }
        let l = load_bytes(&b).unwrap();
        assert_eq!(l.text, "سلام");
        assert_eq!(l.encoding, "utf-16le");
    }

    #[test]
    fn binary_is_rejected() {
        assert_eq!(
            load_bytes(b"\x89PNG\r\n\x1a\n\0\0\0").unwrap_err(),
            LoadError::Binary
        );
    }

    #[test]
    fn windows_1256() {
        // "مرحبا" in Windows-1256.
        let bytes = [0xE3, 0xD1, 0xCD, 0xC8, 0xC7];
        let l = load_bytes(&bytes).unwrap();
        assert_eq!(l.encoding, "windows-1256");
        assert_eq!(l.text, "مرحبا");
    }

    #[test]
    fn size_limit() {
        let dir = std::env::temp_dir().join(format!("nashir-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("big.md");
        std::fs::write(&file, vec![b'a'; 2048]).unwrap();
        let err = load_path(&file, LoadOptions { max_bytes: 1024 }).unwrap_err();
        assert_eq!(err.code(), "too_large");
        let ok = load_path(&file, LoadOptions::default()).unwrap();
        assert_eq!(ok.bytes, 2048);
        assert!(ok.modified_ms > 0);
        assert_eq!(
            load_path(&dir.join("missing.md"), LoadOptions::default()).unwrap_err(),
            LoadError::NotFound
        );
        assert_eq!(
            load_path(&dir, LoadOptions::default()).unwrap_err(),
            LoadError::NotAFile
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
