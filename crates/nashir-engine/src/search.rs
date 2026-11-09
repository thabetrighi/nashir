//! Search within a document, and highlighting of matches.
//!
//! Matching is case-insensitive and Arabic-aware (see [`crate::text`]).
//! Results carry the section they are in, so the reader can scroll to them,
//! and a short snippet with the match, so the results list reads well.

use serde::Serialize;

use crate::direction::fence_marker;
use crate::document::Document;
use crate::text::{normalize, Normalized};

/// One match.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchHit {
    /// The section the match is in.
    pub section: usize,
    /// Text before the match in the snippet.
    pub before: String,
    /// The matched text, as written in the document.
    pub matched: String,
    /// Text after the match in the snippet.
    pub after: String,
    /// The outline entry the match falls under.
    pub heading: Option<usize>,
}

/// The result of a search.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchResult {
    /// The number of matches (up to `limit`).
    pub total: usize,
    /// Whether more matches exist past `limit`.
    pub truncated: bool,
    pub hits: Vec<SearchHit>,
}

const SNIPPET_CHARS: usize = 48;

/// Searches `doc` for `query`; at most `limit` hits are returned.
pub fn search_document(doc: &Document, query: &str, limit: usize) -> SearchResult {
    let needle = normalize(query.trim());
    let mut hits = Vec::new();
    let mut truncated = false;
    if needle.chars().count() == 0 {
        return SearchResult {
            total: 0,
            truncated: false,
            hits,
        };
    }
    'sections: for index in 0..doc.sections.len() {
        let source = doc.section_source(index);
        let normalized = Normalized::new(&source);
        for (start, end) in normalized.find_all(&needle, usize::MAX) {
            if hits.len() >= limit {
                truncated = true;
                break 'sections;
            }
            hits.push(SearchHit {
                section: index,
                before: snippet_before(&source, start),
                matched: source[start..end].to_owned(),
                after: snippet_after(&source, end),
                heading: doc.heading_for_section(index),
            });
        }
    }
    SearchResult {
        total: hits.len(),
        truncated,
        hits,
    }
}

fn clean(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for c in s.chars() {
        let c = if c == '\n' || c == '\t' { ' ' } else { c };
        if matches!(c, '*' | '`' | '#' | '>' | '|') {
            continue;
        }
        if c == ' ' {
            if last_space {
                continue;
            }
            last_space = true;
        } else {
            last_space = false;
        }
        out.push(c);
    }
    out
}

fn snippet_before(source: &str, start: usize) -> String {
    let head = &source[..start];
    let line_start = head.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let chars: Vec<char> = head[line_start..].chars().collect();
    let skip = chars.len().saturating_sub(SNIPPET_CHARS);
    let mut s: String = chars[skip..].iter().collect();
    if skip > 0 {
        s.insert(0, '…');
    }
    clean(&s).trim_start().to_owned()
}

fn snippet_after(source: &str, end: usize) -> String {
    let tail = &source[end..];
    let line_end = tail.find('\n').unwrap_or(tail.len());
    let chars: Vec<char> = tail[..line_end].chars().collect();
    let mut s: String = chars.iter().take(SNIPPET_CHARS).collect();
    if chars.len() > SNIPPET_CHARS {
        s.push('…');
    }
    clean(&s).trim_end().to_owned()
}

/// Byte ranges of `markdown` where a highlight must not be inserted: code
/// (fenced and inline), link targets, autolinks, HTML tags, and heading
/// markers.
fn protected_ranges(markdown: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut pos = 0;
    for raw in markdown.split_inclusive('\n') {
        let start = pos;
        pos += raw.len();
        let line = raw.trim_end_matches('\n');
        let t = line.trim_start();
        if let Some(f) = fence_marker(t) {
            match fence {
                None => fence = Some(f),
                Some((c, n)) if f.0 == c && f.1 >= n => fence = None,
                _ => {}
            }
            out.push((start, pos));
            continue;
        }
        if fence.is_some() || line.starts_with("    ") || line.starts_with('\t') {
            out.push((start, pos));
            continue;
        }
        // Footnote and link definitions.
        if t.starts_with('[') && t.contains("]:") {
            out.push((start, pos));
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'`' => {
                    let n = bytes[i..].iter().take_while(|&&b| b == b'`').count();
                    let ticks = &line[i..i + n];
                    match line[i + n..].find(ticks) {
                        Some(close) => {
                            let end = i + n + close + n;
                            out.push((start + i, start + end));
                            i = end;
                            continue;
                        }
                        None => {
                            i += n;
                            continue;
                        }
                    }
                }
                b']' if bytes.get(i + 1) == Some(&b'(') => {
                    let mut depth = 0i32;
                    let mut j = i + 1;
                    while j < bytes.len() {
                        match bytes[j] {
                            b'(' => depth += 1,
                            b')' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                    let end = (j + 1).min(bytes.len());
                    out.push((start + i, start + end));
                    i = end;
                    continue;
                }
                b'<' => {
                    if let Some(close) = line[i..].find('>') {
                        let inner = &line[i + 1..i + close];
                        if !inner.is_empty() && !inner.contains(' ')
                            || inner.starts_with('/')
                            || inner.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                        {
                            out.push((start + i, start + i + close + 1));
                            i += close + 1;
                            continue;
                        }
                    }
                }
                b'#' if line[..i].trim().is_empty() => {
                    let n = bytes[i..].iter().take_while(|&&b| b == b'#').count();
                    out.push((start + i, start + i + n));
                    i += n;
                    continue;
                }
                b'h' if line[i..].starts_with("http://") || line[i..].starts_with("https://") => {
                    let end = line[i..]
                        .find(|c: char| c.is_whitespace() || c == ')' || c == '>')
                        .map(|e| i + e)
                        .unwrap_or(line.len());
                    out.push((start + i, start + end));
                    i = end;
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
    }
    out
}

/// `markdown` with every match of `query` wrapped in `<mark>` (outside code,
/// link targets and tags), for `TextView`'s `highlight` part. Returns the
/// text and the number of marks inserted.
pub fn highlight_markdown(markdown: &str, query: &str) -> (String, usize) {
    let needle = normalize(query.trim());
    if needle.is_empty() {
        return (markdown.to_owned(), 0);
    }
    let normalized = Normalized::new(markdown);
    let matches = normalized.find_all(&needle, usize::MAX);
    if matches.is_empty() {
        return (markdown.to_owned(), 0);
    }
    let protected = protected_ranges(markdown);
    let mut out = String::with_capacity(markdown.len() + matches.len() * 13);
    let mut last = 0;
    let mut count = 0;
    let mut p = 0;
    for (s, e) in matches {
        while p < protected.len() && protected[p].1 <= s {
            p += 1;
        }
        let blocked = protected[p..]
            .iter()
            .take_while(|r| r.0 < e)
            .any(|r| r.0 < e && r.1 > s);
        // A match that runs over a line break would break block syntax.
        if blocked || markdown[s..e].contains('\n') {
            continue;
        }
        out.push_str(&markdown[last..s]);
        out.push_str("<mark>");
        out.push_str(&markdown[s..e]);
        out.push_str("</mark>");
        last = e;
        count += 1;
    }
    out.push_str(&markdown[last..]);
    (out, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_arabic_without_diacritics() {
        let d = Document::parse("# مقدمة\n\nقرأ أَحْمَدُ الكتاب.\n\n## ثانياً\n\nثم عاد احمد.\n".to_owned());
        let r = search_document(&d, "احمد", 100);
        assert_eq!(r.total, 2);
        assert_eq!(r.hits[0].section, 0);
        assert_eq!(r.hits[0].matched, "أَحْمَدُ".trim_end_matches('\u{064F}'));
        assert_eq!(r.hits[1].section, 1);
        assert_eq!(r.hits[1].heading, Some(1));
        assert!(r.hits[1].before.contains("ثم عاد"));
    }

    #[test]
    fn limit_truncates() {
        let d = Document::parse("a a a a a".to_owned());
        let r = search_document(&d, "A", 3);
        assert_eq!(r.total, 3);
        assert!(r.truncated);
    }

    #[test]
    fn highlight_skips_code_and_links() {
        let md = "Rust is fast. `rust` code and [Rust](https://rust-lang.org/rust) <span class=\"rust\">RUST</span>\n\n```\nrust\n```\n";
        let (out, n) = highlight_markdown(md, "rust");
        assert_eq!(n, 3, "{out}");
        assert!(out.starts_with("<mark>Rust</mark> is fast. `rust`"));
        assert!(out.contains("[<mark>Rust</mark>](https://rust-lang.org/rust)"));
        assert!(out.contains("<span class=\"rust\"><mark>RUST</mark></span>"));
        assert!(out.contains("```\nrust\n```"));
    }

    #[test]
    fn highlight_heading_keeps_marker() {
        let (out, n) = highlight_markdown("## Install guide\n", "install");
        assert_eq!(n, 1);
        assert_eq!(out, "## <mark>Install</mark> guide\n");
    }

    #[test]
    fn empty_query() {
        let d = Document::parse("abc".to_owned());
        assert_eq!(search_document(&d, "  ", 10).total, 0);
        assert_eq!(highlight_markdown("abc", "").1, 0);
    }
}
