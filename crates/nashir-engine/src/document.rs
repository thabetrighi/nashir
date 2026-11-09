//! Splitting a Markdown document into sections.
//!
//! NSH's `TextView` renders Markdown natively. Rendering a 10 MB document
//! as one `TextView` would build every paragraph up front, so Nashir splits
//! the document into **sections** that a virtualized `List` renders lazily:
//!
//! - every heading (ATX `#` or setext `===` / `---`) starts a section, so the
//!   outline maps each heading to exactly one list row;
//! - long runs without headings are cut at a blank line once a section passes
//!   [`SOFT_LIMIT`] bytes;
//! - a gigantic code block is cut past [`HARD_LIMIT`] bytes: the fence is
//!   closed at the end of one section and reopened (with its info string) at
//!   the start of the next, so each piece is still valid Markdown.
//!
//! Footnote definitions and link reference definitions are document-wide
//! in Markdown; a section rendered alone would lose them. Each section's
//! Markdown therefore gets the definitions it references appended.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use serde::Serialize;

use crate::direction::{count_strong, direction_from_counts, fence_marker, Direction};
use crate::slug::{heading_text, Slugger};
use crate::text::word_count;

/// Sections without headings are cut at a blank line past this many bytes.
pub const SOFT_LIMIT: usize = 24 * 1024;
/// A code block is cut (closed and reopened) past this many bytes.
pub const HARD_LIMIT: usize = 96 * 1024;

/// A renderable piece of the document.
#[derive(Debug, Clone)]
pub struct Section {
    /// Byte ranges of the source that make up the section, in order
    /// (several when footnote definitions were taken out of it).
    pub pieces: Vec<Range<usize>>,
    /// Text put before the pieces (a reopened code fence).
    pub prefix: Option<String>,
    /// Text put after the pieces (a closed code fence).
    pub suffix: Option<String>,
    /// The outline entry this section starts with, if it starts with a heading.
    pub heading: Option<usize>,
    pub direction: Direction,
    pub bytes: usize,
}

/// A heading of the document.
#[derive(Debug, Clone, Serialize)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub anchor: String,
    pub section: usize,
}

/// Figures shown in the status bar.
#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Stats {
    pub words: usize,
    pub characters: usize,
    pub lines: usize,
    pub headings: usize,
    pub code_blocks: usize,
    pub diagrams: usize,
    pub reading_minutes: usize,
}

/// A parsed document, keeping its source and the sections over it.
#[derive(Debug, Clone)]
pub struct Document {
    pub source: String,
    pub sections: Vec<Section>,
    pub outline: Vec<OutlineItem>,
    pub direction: Direction,
    pub stats: Stats,
    footnotes: HashMap<String, Range<usize>>,
    link_definitions: String,
    has_front_matter: bool,
}

struct Builder<'a> {
    src: &'a str,
    sections: Vec<Section>,
    current_start: usize,
    current_pieces: Vec<Range<usize>>,
    current_prefix: Option<String>,
    current_heading: Option<usize>,
    /// Strong RTL and LTR characters over all sections.
    totals: (usize, usize),
}

impl<'a> Builder<'a> {
    /// Ends the current section at byte `at` (exclusive).
    fn cut(&mut self, at: usize, suffix: Option<String>) {
        if at > self.current_start {
            self.current_pieces.push(self.current_start..at);
        }
        let bytes: usize = self.current_pieces.iter().map(|r| r.len()).sum();
        let has_content = self
            .current_pieces
            .iter()
            .any(|r| !self.src[r.clone()].trim().is_empty())
            || self.current_prefix.is_some();
        if has_content {
            let (rtl, ltr) = self
                .current_pieces
                .iter()
                .map(|r| count_strong(&self.src[r.clone()]))
                .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            self.totals.0 += rtl;
            self.totals.1 += ltr;
            self.sections.push(Section {
                pieces: std::mem::take(&mut self.current_pieces),
                prefix: self.current_prefix.take(),
                suffix,
                heading: self.current_heading.take(),
                direction: direction_from_counts(rtl, ltr),
                bytes,
            });
        } else {
            self.current_pieces.clear();
            self.current_prefix = None;
            // A heading never sits in an empty section: keep it for the next.
        }
        self.current_start = at;
    }

    fn current_len(&self, at: usize) -> usize {
        self.current_pieces.iter().map(|r| r.len()).sum::<usize>() + at.saturating_sub(self.current_start)
    }

    /// Takes `range` out of the current section (a footnote definition).
    fn exclude(&mut self, range: Range<usize>) {
        if range.start > self.current_start {
            self.current_pieces.push(self.current_start..range.start);
        }
        self.current_start = range.end;
    }
}

fn atx_heading(line: &str) -> Option<(u8, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let t = &line[indent..];
    let level = t.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &t[level..];
    if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }
    // Strip an optional closing sequence of #.
    let mut content = rest.trim();
    let stripped = content.trim_end_matches('#');
    if stripped.len() != content.len() && (stripped.is_empty() || stripped.ends_with(' ')) {
        content = stripped.trim_end();
    }
    Some((level as u8, content))
}

fn setext_underline(line: &str) -> Option<u8> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let t = line.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars().all(|c| c == '=') {
        Some(1)
    } else if t.chars().all(|c| c == '-') {
        Some(2)
    } else {
        None
    }
}

/// Whether a line can be the text of a setext heading (a paragraph line).
fn paragraph_like(line: &str) -> bool {
    let t = line.trim_start();
    if t.is_empty() || line.len() - t.len() > 3 {
        return false;
    }
    let first = t.chars().next().unwrap();
    if matches!(first, '>' | '#' | '|' | '<') {
        return false;
    }
    if (first == '-' || first == '*' || first == '+') && t[1..].starts_with(' ') {
        return false;
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && (t[digits..].starts_with(". ") || t[digits..].starts_with(") ")) {
        return false;
    }
    !t.contains('|') || !t.trim_end().ends_with('|')
}

fn footnote_definition_label(line: &str) -> Option<&str> {
    let t = line.trim_start_matches(' ');
    if line.len() - t.len() > 3 {
        return None;
    }
    let rest = t.strip_prefix("[^")?;
    let close = rest.find("]:")?;
    let label = &rest[..close];
    (!label.is_empty() && !label.contains(']')).then_some(label)
}

fn is_link_definition(line: &str) -> bool {
    let t = line.trim_start_matches(' ');
    if line.len() - t.len() > 3 || !t.starts_with('[') || t.starts_with("[^") {
        return false;
    }
    match t.find("]:") {
        Some(i) => i > 1 && !t[1..i].contains(']'),
        None => false,
    }
}

/// Iterates the lines of `src` with their byte range (newline included).
fn lines_with_offsets(src: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    let mut pos = 0;
    std::iter::from_fn(move || {
        if pos >= src.len() {
            return None;
        }
        let start = pos;
        let end = match src[start..].find('\n') {
            Some(i) => start + i + 1,
            None => src.len(),
        };
        pos = end;
        let line = src[start..end].trim_end_matches('\n');
        Some((start, end, line))
    })
}

impl Document {
    /// Parses `source` into sections and an outline.
    pub fn parse(source: String) -> Document {
        let src = source.as_str();
        let mut b = Builder {
            src,
            sections: Vec::new(),
            current_start: 0,
            current_pieces: Vec::new(),
            current_prefix: None,
            current_heading: None,
            totals: (0, 0),
        };
        let mut outline: Vec<OutlineItem> = Vec::new();
        let mut slugger = Slugger::default();
        let mut footnotes: HashMap<String, Range<usize>> = HashMap::new();
        let mut link_definitions = String::new();
        let mut stats = Stats::default();

        let lines: Vec<(usize, usize, &str)> = lines_with_offsets(src).collect();
        stats.lines = lines.len();

        // Front matter: `---` on the first line, closed by `---` or `...`.
        let mut i = 0;
        let mut has_front_matter = false;
        if lines.first().is_some_and(|l| l.2.trim_end() == "---") {
            if let Some(close) = lines
                .iter()
                .take(400)
                .skip(1)
                .position(|l| matches!(l.2.trim_end(), "---" | "..."))
            {
                has_front_matter = true;
                i = close + 2;
            }
        }

        let mut fence: Option<(char, usize, String)> = None; // char, length, opening line
        let mut prev_paragraph_start: Option<usize> = None;

        while i < lines.len() {
            let (start, end, line) = lines[i];
            let trimmed = line.trim_start();

            // Inside a code block.
            if let Some((fc, flen, opening)) = fence.clone() {
                if let Some((c, n)) = fence_marker(trimmed) {
                    if c == fc && n >= flen && trimmed.trim_start_matches(c).trim().is_empty() {
                        fence = None;
                        i += 1;
                        continue;
                    }
                }
                if b.current_len(start) > HARD_LIMIT {
                    let close = format!("{}\n", fc.to_string().repeat(flen));
                    b.cut(start, Some(close));
                    b.current_prefix = Some(format!("{opening}\n"));
                }
                let _ = opening;
                i += 1;
                continue;
            }

            if let Some((c, n)) = fence_marker(trimmed) {
                let info = trimmed.trim_start_matches(c).trim();
                // A backtick fence's info string cannot contain backticks.
                if c == '~' || !info.contains('`') {
                    stats.code_blocks += 1;
                    if info
                        .split_whitespace()
                        .next()
                        .is_some_and(|l| l.eq_ignore_ascii_case("mermaid"))
                    {
                        stats.diagrams += 1;
                    }
                    fence = Some((c, n, trimmed.trim_end().to_owned()));
                    prev_paragraph_start = None;
                    i += 1;
                    continue;
                }
            }

            if let Some((level, content)) = atx_heading(line) {
                b.cut(start, None);
                let text = heading_text(content);
                outline.push(OutlineItem {
                    level,
                    anchor: slugger.slug(&text),
                    text,
                    section: b.sections.len(),
                });
                b.current_heading = Some(outline.len() - 1);
                prev_paragraph_start = None;
                i += 1;
                continue;
            }

            if let (Some(level), Some(pstart)) = (setext_underline(line), prev_paragraph_start) {
                // The heading is the previous line (a one-line paragraph).
                let heading_line = lines[i - 1].2;
                b.cut(pstart, None);
                let text = heading_text(heading_line.trim());
                outline.push(OutlineItem {
                    level,
                    anchor: slugger.slug(&text),
                    text,
                    section: b.sections.len(),
                });
                b.current_heading = Some(outline.len() - 1);
                prev_paragraph_start = None;
                i += 1;
                continue;
            }

            if let Some(label) = footnote_definition_label(line) {
                // The definition: this line and the indented lines after it.
                let mut j = i + 1;
                while j < lines.len() {
                    let l = lines[j].2;
                    if l.trim().is_empty() {
                        if lines
                            .get(j + 1)
                            .is_some_and(|n| n.2.starts_with("    ") || n.2.starts_with('\t'))
                        {
                            j += 1;
                            continue;
                        }
                        break;
                    }
                    if l.starts_with("    ") || l.starts_with('\t') || l.starts_with("  ") {
                        j += 1;
                    } else {
                        break;
                    }
                }
                let def_end = lines[j - 1].1;
                footnotes.insert(label.to_owned(), start..def_end);
                b.exclude(start..def_end);
                prev_paragraph_start = None;
                i = j;
                continue;
            }

            if is_link_definition(line) {
                link_definitions.push_str(line.trim());
                link_definitions.push('\n');
            }

            if line.trim().is_empty() {
                prev_paragraph_start = None;
                if b.current_len(end) > SOFT_LIMIT {
                    b.cut(end, None);
                }
            } else if paragraph_like(line) && prev_paragraph_start.is_none() {
                prev_paragraph_start = Some(start);
            } else if !paragraph_like(line) {
                prev_paragraph_start = None;
            } else {
                // A second paragraph line: a setext underline after it
                // would make the whole paragraph a heading; keep it simple
                // and only treat one-line paragraphs as setext headings.
                prev_paragraph_start = Some(start);
            }
            i += 1;
        }
        let suffix = fence.map(|(c, n, _)| format!("\n{}\n", c.to_string().repeat(n)));
        b.cut(src.len(), suffix);

        if b.sections.is_empty() {
            b.sections.push(Section {
                #[allow(clippy::single_range_in_vec_init)]
                pieces: vec![0..src.len()],
                prefix: None,
                suffix: None,
                heading: None,
                direction: Direction::Ltr,
                bytes: src.len(),
            });
        }

        let (rtl, ltr) = b.totals;
        let direction = direction_from_counts(rtl, ltr);
        stats.words = word_count(src);
        stats.characters = src.chars().count();
        stats.headings = outline.len();
        stats.reading_minutes = stats.words.div_ceil(200).max(1);

        Document {
            sections: b.sections,
            outline,
            direction,
            stats,
            footnotes,
            link_definitions,
            has_front_matter,
            source,
        }
    }

    /// Whether the document starts with YAML front matter.
    pub fn has_front_matter(&self) -> bool {
        self.has_front_matter
    }

    /// The raw source of section `index` (without appended definitions).
    pub fn section_source(&self, index: usize) -> String {
        let Some(section) = self.sections.get(index) else {
            return String::new();
        };
        let mut out = String::with_capacity(section.bytes + 16);
        if let Some(p) = &section.prefix {
            out.push_str(p);
        }
        for r in &section.pieces {
            out.push_str(&self.source[r.clone()]);
        }
        if let Some(s) = &section.suffix {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(s);
        }
        out
    }

    /// The Markdown that renders section `index` on its own: its source,
    /// then the footnote definitions it references and, when it has links,
    /// the document's link reference definitions.
    pub fn section_markdown(&self, index: usize) -> String {
        let source = self.section_source(index);
        self.complete(source)
    }

    /// Appends the definitions `markdown` references (used for highlighted
    /// copies of a section too).
    pub fn complete(&self, mut markdown: String) -> String {
        let refs = footnote_references(&markdown);
        let mut appended = HashSet::new();
        let mut extra = String::new();
        for label in refs {
            if appended.insert(label.clone()) {
                if let Some(range) = self.footnotes.get(&label) {
                    extra.push_str(self.source[range.clone()].trim_end());
                    extra.push_str("\n\n");
                }
            }
        }
        if !self.link_definitions.is_empty()
            && self.link_definitions.len() < 64 * 1024
            && markdown.contains(']')
        {
            extra.push_str(&self.link_definitions);
        }
        if !extra.is_empty() {
            if !markdown.ends_with("\n\n") {
                markdown.push_str(if markdown.ends_with('\n') { "\n" } else { "\n\n" });
            }
            markdown.push_str(&extra);
        }
        markdown
    }

    /// The section that holds the heading with `anchor` (`#` optional).
    pub fn section_for_anchor(&self, anchor: &str) -> Option<usize> {
        let anchor = anchor.trim_start_matches('#');
        let decoded = percent_decode(anchor);
        self.outline
            .iter()
            .find(|o| o.anchor == anchor || o.anchor == decoded)
            .map(|o| o.section)
    }

    /// The outline entry that is current while section `index` is at the top:
    /// the last heading at or before it.
    pub fn heading_for_section(&self, index: usize) -> Option<usize> {
        let mut found = None;
        for (i, o) in self.outline.iter().enumerate() {
            if o.section <= index {
                found = Some(i);
            } else {
                break;
            }
        }
        found
    }
}

/// The labels of footnote references (`[^label]`) in `markdown`, skipping
/// definitions and code.
fn footnote_references(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence: Option<(char, usize)> = None;
    for line in markdown.lines() {
        let t = line.trim_start();
        if let Some(f) = fence_marker(t) {
            match in_fence {
                None => in_fence = Some(f),
                Some((c, n)) if f.0 == c && f.1 >= n => in_fence = None,
                _ => {}
            }
            continue;
        }
        if in_fence.is_some() {
            continue;
        }
        let mut rest = line;
        let mut first = true;
        while let Some(pos) = rest.find("[^") {
            let after = &rest[pos + 2..];
            if let Some(close) = after.find(']') {
                let label = &after[..close];
                let is_def = first && rest[..pos].trim().is_empty() && after[close + 1..].starts_with(':');
                if !label.is_empty() && !is_def && !label.contains('[') {
                    out.push(label.to_owned());
                }
                rest = &after[close + 1..];
            } else {
                break;
            }
            first = false;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# العنوان\n\nمقدمة قصيرة[^1].\n\n## Install\n\n```bash\n# not a heading\nnpm i\n```\n\nSetext\n======\n\nText with [a ref][r].\n\n[r]: https://example.com\n\n[^1]: حاشية مهمة.\n";

    #[test]
    fn headings_start_sections() {
        let d = Document::parse(DOC.to_owned());
        let texts: Vec<_> = d.outline.iter().map(|o| (o.level, o.text.as_str())).collect();
        assert_eq!(texts, vec![(1, "العنوان"), (2, "Install"), (1, "Setext")]);
        assert_eq!(d.outline[0].section, 0);
        assert_eq!(d.outline[1].section, 1);
        assert_eq!(d.outline[2].section, 2);
        assert!(d.section_source(1).contains("# not a heading"));
        assert_eq!(d.stats.code_blocks, 1);
    }

    #[test]
    fn footnotes_follow_their_references() {
        let d = Document::parse(DOC.to_owned());
        let first = d.section_markdown(0);
        assert!(first.contains("[^1]: حاشية مهمة."), "{first}");
        // The definition is taken out of the section it was written in.
        assert!(!d.section_source(2).contains("[^1]:"));
    }

    #[test]
    fn link_definitions_are_shared() {
        let d = Document::parse(DOC.to_owned());
        assert!(d.section_markdown(2).contains("[r]: https://example.com"));
    }

    #[test]
    fn anchors() {
        let d = Document::parse(DOC.to_owned());
        assert_eq!(d.section_for_anchor("#install"), Some(1));
        assert_eq!(d.section_for_anchor("العنوان"), Some(0));
        assert_eq!(
            d.section_for_anchor("%D8%A7%D9%84%D8%B9%D9%86%D9%88%D8%A7%D9%86"),
            Some(0)
        );
        assert_eq!(d.heading_for_section(2), Some(2));
    }

    #[test]
    fn direction_is_detected() {
        let d = Document::parse(DOC.to_owned());
        assert_eq!(d.sections[0].direction, Direction::Rtl);
        assert_eq!(d.sections[1].direction, Direction::Ltr);
    }

    #[test]
    fn long_text_is_split_at_blank_lines() {
        let para = "word ".repeat(200) + "\n\n";
        let src = para.repeat(200);
        let d = Document::parse(src.clone());
        assert!(d.sections.len() > 5);
        for s in &d.sections {
            assert!(s.bytes <= SOFT_LIMIT + para.len());
        }
        let joined: String = (0..d.sections.len()).map(|i| d.section_source(i)).collect();
        assert_eq!(joined, src);
    }

    #[test]
    fn huge_code_blocks_are_closed_and_reopened() {
        let mut src = String::from("```rust title=x\n");
        for i in 0..20_000 {
            src.push_str(&format!("let value_{i} = {i};\n"));
        }
        src.push_str("```\n");
        let d = Document::parse(src);
        assert!(d.sections.len() > 2);
        for i in 0..d.sections.len() {
            let md = d.section_source(i);
            assert!(md.starts_with("```rust title=x\n"), "section {i}");
            assert!(md.trim_end().ends_with("```"), "section {i}");
        }
    }

    #[test]
    fn front_matter_is_not_a_heading() {
        let d = Document::parse("---\ntitle: x\n---\n\nText\n---\n".to_owned());
        assert!(d.has_front_matter());
        assert_eq!(d.outline.len(), 1); // "Text" with a setext underline
    }

    #[test]
    fn empty_document_has_one_section() {
        let d = Document::parse(String::new());
        assert_eq!(d.sections.len(), 1);
    }
}
