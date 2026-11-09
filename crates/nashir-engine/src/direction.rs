//! Base direction detection.
//!
//! A document is right-to-left when a meaningful share of its *strong*
//! characters belong to RTL scripts (Arabic, Hebrew, Syriac, Thaana, N'Ko…).
//! Code (fenced and inline), URLs and HTML tags are skipped: an Arabic
//! technical document full of English code is still an Arabic document.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Ltr,
    Rtl,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        }
    }
}

/// Whether `c` is a strong right-to-left character.
pub fn is_rtl_char(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x08FF      // Hebrew, Arabic, Syriac, Thaana, N'Ko, Samaritan, Arabic Ext-A/B
        | 0xFB1D..=0xFDFF    // Hebrew & Arabic presentation forms A
        | 0xFE70..=0xFEFF    // Arabic presentation forms B
        | 0x10800..=0x10FFF  // Historic RTL scripts
        | 0x1E800..=0x1EFFF) // Mende Kikakui, Adlam, Arabic mathematical symbols
        && !is_neutral_in_rtl_block(c)
}

/// Arabic-Indic digits and some punctuation inside RTL blocks are weak or
/// neutral; they do not decide the direction.
fn is_neutral_in_rtl_block(c: char) -> bool {
    matches!(c as u32, 0x0660..=0x0669 | 0x06F0..=0x06F9 | 0x064B..=0x065F | 0x0670 | 0x0640)
}

/// Whether `c` is a strong left-to-right character (a letter of an LTR script).
pub fn is_ltr_char(c: char) -> bool {
    c.is_alphabetic()
        && !is_rtl_char(c)
        && !is_neutral_in_rtl_block(c)
        && !matches!(c as u32, 0x0590..=0x08FF)
}

/// Counts strong characters in `text`, skipping code, URLs and tags.
pub fn count_strong(text: &str) -> (usize, usize) {
    let mut rtl = 0usize;
    let mut ltr = 0usize;
    let mut in_fence: Option<(char, usize)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(fence) = fence_marker(trimmed) {
            match in_fence {
                None => in_fence = Some(fence),
                Some((ch, n)) if fence.0 == ch && fence.1 >= n => in_fence = None,
                _ => {}
            }
            continue;
        }
        if in_fence.is_some() {
            continue;
        }
        let mut in_code = false;
        let mut skip_until_space = false;
        let mut in_tag = false;
        let bytes = line.as_bytes();
        for (i, c) in line.char_indices() {
            if c == '`' {
                in_code = !in_code;
            } else if in_code {
            } else if skip_until_space {
                if c.is_whitespace() || c == ')' || c == '>' {
                    skip_until_space = false;
                }
            } else if in_tag {
                if c == '>' {
                    in_tag = false;
                }
            } else if c.is_ascii() {
                if c == '<'
                    && bytes
                        .get(i + 1)
                        .is_some_and(|n| n.is_ascii_alphabetic() || *n == b'/')
                {
                    in_tag = true;
                } else if (c == 'h'
                    && (line[i..].starts_with("http://") || line[i..].starts_with("https://")))
                    || (c == ']' && bytes.get(i + 1) == Some(&b'('))
                {
                    skip_until_space = true;
                } else if c.is_ascii_alphabetic() {
                    ltr += 1;
                }
            } else if is_rtl_char(c) {
                rtl += 1;
            } else if is_ltr_char(c) {
                ltr += 1;
            }
        }
    }
    (rtl, ltr)
}

/// A fence opener/closer: its character and length.
pub(crate) fn fence_marker(trimmed: &str) -> Option<(char, usize)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let n = trimmed.chars().take_while(|&c| c == first).count();
    (n >= 3).then_some((first, n))
}

/// The base direction of `text`: RTL when at least 30% of the strong
/// characters are RTL (Arabic technical writing mixes in many Latin terms).
pub fn detect_direction(text: &str) -> Direction {
    let (rtl, ltr) = count_strong(text);
    direction_from_counts(rtl, ltr)
}

pub fn direction_from_counts(rtl: usize, ltr: usize) -> Direction {
    if rtl == 0 {
        Direction::Ltr
    } else if rtl * 10 >= (rtl + ltr) * 3 {
        Direction::Rtl
    } else {
        Direction::Ltr
    }
}

/// The direction of a single label: the first strong character decides,
/// as the Unicode bidi algorithm's rules P2–P3 do.
pub fn first_strong(text: &str) -> Option<Direction> {
    text.chars().find_map(|c| {
        if is_rtl_char(c) {
            Some(Direction::Rtl)
        } else if is_ltr_char(c) {
            Some(Direction::Ltr)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_document_with_code_is_rtl() {
        let doc = "# دليل التثبيت\n\nقم بتشغيل الأمر التالي لتثبيت الحزمة:\n\n```bash\nnpm install --save my-very-long-package-name\n```\n\nثم افتح الملف `src/main.rs` وعدّل الإعدادات.";
        assert_eq!(detect_direction(doc), Direction::Rtl);
    }

    #[test]
    fn english_document_is_ltr() {
        assert_eq!(
            detect_direction("# Install\n\nRun the command below."),
            Direction::Ltr
        );
    }

    #[test]
    fn mixed_arabic_with_english_terms() {
        // Many Latin terms, still mostly Arabic prose.
        let doc = "يستخدم التطبيق Rust و TypeScript مع إطار Native Shell لعرض ملفات Markdown بسرعة.";
        assert_eq!(detect_direction(doc), Direction::Rtl);
    }

    #[test]
    fn urls_and_tags_are_ignored() {
        let doc = "رابط: https://example.com/a/very/long/english/path/that/should/not/count <span class=\"english\">x</span>";
        assert_eq!(detect_direction(doc), Direction::Rtl);
    }

    #[test]
    fn first_strong_char() {
        assert_eq!(first_strong("123 مرحبا hello"), Some(Direction::Rtl));
        assert_eq!(first_strong("  hello مرحبا"), Some(Direction::Ltr));
        assert_eq!(first_strong("123 ٤٥"), None);
    }
}
