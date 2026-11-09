//! GitHub-style heading anchors (`## Getting started` → `getting-started`),
//! the same anchors NSH's `TextView` reports for links such as `#getting-started`.

use std::collections::HashMap;

/// The plain text of an inline Markdown heading: emphasis, code marks and
/// link targets removed, `[text](url)` → `text`, HTML tags dropped.
pub fn heading_text(markdown: &str) -> String {
    let chars: Vec<char> = markdown.chars().collect();
    let mut out = String::with_capacity(markdown.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if i + 1 < chars.len() => {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            '*' | '_' | '`' | '~' => {}
            '!' if chars.get(i + 1) == Some(&'[') => {}
            '[' | ']' => {
                if c == ']' && chars.get(i + 1) == Some(&'(') {
                    // Skip the link target.
                    let mut depth = 0;
                    i += 1;
                    while i < chars.len() {
                        match chars[i] {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            '<' => {
                if let Some(close) = chars[i..].iter().position(|&x| x == '>') {
                    let inner: String = chars[i + 1..i + close].iter().collect();
                    if inner.starts_with('/') || inner.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                    {
                        i += close + 1;
                        continue;
                    }
                }
                out.push(c);
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out.trim().to_owned()
}

/// GitHub's slug: lower case, letters/digits/marks kept, spaces → `-`,
/// other punctuation removed.
pub fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.trim().chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' || is_mark(c) {
            for l in c.to_lowercase() {
                out.push(l);
            }
        } else if c == ' ' {
            out.push('-');
        }
    }
    out
}

fn is_mark(c: char) -> bool {
    // Combining marks, including Arabic tashkeel.
    matches!(c as u32, 0x0300..=0x036F | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06ED)
}

/// Gives repeated slugs GitHub's suffixes: `a`, `a-1`, `a-2`.
#[derive(Default)]
pub struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let base = slugify(text);
        match self.seen.get_mut(&base) {
            None => {
                self.seen.insert(base.clone(), 0);
                base
            }
            Some(n) => {
                *n += 1;
                let candidate = format!("{base}-{n}");
                self.seen.insert(candidate.clone(), 0);
                candidate
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_slugs() {
        assert_eq!(slugify("Getting Started"), "getting-started");
        assert_eq!(slugify("What's new in v2.0?"), "whats-new-in-v20");
        assert_eq!(slugify("دعم اللغة العربية"), "دعم-اللغة-العربية");
        assert_eq!(slugify("C++ & Rust"), "c--rust");
    }

    #[test]
    fn heading_markup_is_removed() {
        assert_eq!(
            heading_text("**Bold** `code` and [a link](http://x.y/(z))"),
            "Bold code and a link"
        );
        assert_eq!(heading_text("مقدمة <a id=\"x\"></a>"), "مقدمة");
    }

    #[test]
    fn duplicates() {
        let mut s = Slugger::default();
        assert_eq!(s.slug("Intro"), "intro");
        assert_eq!(s.slug("Intro"), "intro-1");
        assert_eq!(s.slug("Intro"), "intro-2");
    }
}
