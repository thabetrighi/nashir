//! Arabic-aware text normalisation for search.
//!
//! Readers rarely type a word exactly as it was written: diacritics
//! (tashkeel) are usually left out, hamza forms of alef vary, final yaa and
//! alef maqsura are used interchangeably, and so are taa marbuta and haa.
//! Search therefore compares *normalised* text, while keeping a map back to
//! the original byte offsets so matches can be highlighted in place.

/// Normalises one character for matching. Returns `None` to drop it.
///
/// - Latin and other cased scripts are lower-cased (search is case-insensitive);
/// - Arabic diacritics (U+064B–U+065F, U+0670) and tatweel (U+0640) are dropped;
/// - أ إ آ ٱ → ا, ى → ي, ة → ه, ؤ → و, ئ → ي;
/// - Arabic-Indic and Eastern Arabic-Indic digits → ASCII digits;
/// - zero-width joiners and direction marks are dropped.
pub fn normalize_char(c: char, out: &mut String) {
    match c as u32 {
        0x064B..=0x065F | 0x0670 | 0x0640 => {}
        0x200B..=0x200F | 0x202A..=0x202E | 0x2066..=0x2069 | 0xFEFF => {}
        0x0622 | 0x0623 | 0x0625 | 0x0671 | 0x0672 | 0x0673 => out.push('\u{0627}'),
        0x0649 => out.push('\u{064A}'),
        0x0629 => out.push('\u{0647}'),
        0x0624 => out.push('\u{0648}'),
        0x0626 => out.push('\u{064A}'),
        0x0660..=0x0669 => out.push(char::from(b'0' + (c as u32 - 0x0660) as u8)),
        0x06F0..=0x06F9 => out.push(char::from(b'0' + (c as u32 - 0x06F0) as u8)),
        _ => {
            if c.is_uppercase() {
                for l in c.to_lowercase() {
                    out.push(l);
                }
            } else {
                out.push(c);
            }
        }
    }
}

/// Normalises a whole string (used for queries).
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        normalize_char(c, &mut out);
    }
    out
}

/// A normalised copy of a text with, for each byte of the normalised text,
/// the byte range of the original character it came from.
pub struct Normalized {
    pub text: String,
    /// `starts[i]` / `ends[i]`: original byte range of normalised byte `i`.
    starts: Vec<u32>,
    ends: Vec<u32>,
}

impl Normalized {
    pub fn new(original: &str) -> Self {
        let mut text = String::with_capacity(original.len());
        let mut starts = Vec::with_capacity(original.len());
        let mut ends = Vec::with_capacity(original.len());
        let mut buf = String::new();
        for (i, c) in original.char_indices() {
            buf.clear();
            normalize_char(c, &mut buf);
            let end = (i + c.len_utf8()) as u32;
            for _ in 0..buf.len() {
                starts.push(i as u32);
                ends.push(end);
            }
            text.push_str(&buf);
        }
        Normalized { text, starts, ends }
    }

    /// The original byte range of the normalised byte range `start..end`.
    pub fn original_range(&self, start: usize, end: usize) -> (usize, usize) {
        let s = self.starts[start] as usize;
        let e = self.ends[end - 1] as usize;
        (s, e)
    }

    /// Every match of the normalised `needle` (non-overlapping), as
    /// original byte ranges.
    pub fn find_all(&self, needle: &str, limit: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        if needle.is_empty() {
            return out;
        }
        let mut from = 0;
        while let Some(pos) = self.text[from..].find(needle) {
            let start = from + pos;
            let end = start + needle.len();
            out.push(self.original_range(start, end));
            if out.len() >= limit {
                break;
            }
            from = end;
        }
        out
    }
}

/// Counts words: runs of letters or digits.
pub fn word_count(text: &str) -> usize {
    let mut count = 0;
    let mut in_word = false;
    for c in text.chars() {
        let w = c.is_alphanumeric() || matches!(c as u32, 0x064B..=0x065F | 0x0670 | 0x0640);
        if w && !in_word {
            count += 1;
        }
        in_word = w;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_variants_match() {
        assert_eq!(normalize("أَحْمَد"), normalize("احمد"));
        assert_eq!(normalize("مدرسة"), normalize("مدرسه"));
        assert_eq!(normalize("على"), normalize("علي"));
        assert_eq!(normalize("إسلام"), normalize("اسلام"));
        assert_eq!(normalize("آمن"), normalize("امن"));
        assert_eq!(normalize("مـــرحبا"), normalize("مرحبا"));
        assert_eq!(normalize("٢٠٢٦"), "2026");
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(normalize("Native SHELL"), "native shell");
    }

    #[test]
    fn find_maps_back_to_original() {
        let original = "قال أَحْمَدُ: Hello WORLD";
        let n = Normalized::new(original);
        let hits = n.find_all(&normalize("احمد"), 10);
        assert_eq!(hits.len(), 1);
        let (s, e) = hits[0];
        assert_eq!(&original[s..e], "أَحْمَد");
        let hits = n.find_all(&normalize("world"), 10);
        assert_eq!(&original[hits[0].0..hits[0].1], "WORLD");
    }

    #[test]
    fn words() {
        assert_eq!(word_count("مرحبا بالعالم، hello world 2026"), 5);
    }
}
