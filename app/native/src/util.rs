//! Small helpers: running work off the engine thread, base64, file URLs.

use std::path::{Path, PathBuf};

/// Runs `work` on a new thread and resolves with its result, so a large
/// file is read and parsed without blocking the engine thread (the UI keeps
/// updating while a 50 MB document loads).
pub async fn background<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = futures_channel::oneshot::channel();
    std::thread::Builder::new()
        .name("nashir-worker".into())
        .spawn(move || {
            let _ = tx.send(work());
        })
        .map_err(|e| format!("io:{e}"))?;
    rx.await.map_err(|_| "io:worker stopped".to_owned())
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A `file:` URL (as a file drop gives them) or a plain path → a path.
pub fn path_from_uri(uri: &str) -> PathBuf {
    let Some(rest) = uri.strip_prefix("file://") else {
        return PathBuf::from(uri);
    };
    // file:///C:/x (Windows), file:///home/x (Unix), file://server/share (UNC).
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    let decoded = percent_decode(rest);
    if cfg!(windows) {
        let trimmed = decoded.trim_start_matches('/');
        let is_drive = trimmed.as_bytes().get(1) == Some(&b':');
        if is_drive {
            PathBuf::from(trimmed.replace('/', "\\"))
        } else {
            PathBuf::from(format!(
                "\\\\{}",
                decoded.trim_start_matches('/').replace('/', "\\")
            ))
        }
    } else {
        PathBuf::from(decoded)
    }
}

/// The MIME type of an image file, by extension.
pub fn image_mime(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "heic" => "image/heic",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn file_urls() {
        if cfg!(windows) {
            assert_eq!(
                path_from_uri("file:///C:/My%20Docs/%D9%86.md"),
                PathBuf::from("C:\\My Docs\\ن.md")
            );
        } else {
            assert_eq!(
                path_from_uri("file:///home/me/My%20Docs/%D9%86.md"),
                PathBuf::from("/home/me/My Docs/ن.md")
            );
        }
        assert_eq!(path_from_uri("relative.md"), PathBuf::from("relative.md"));
    }
}
