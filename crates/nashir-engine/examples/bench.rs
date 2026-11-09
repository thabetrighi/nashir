//! Measures the engine on generated documents of 100 KB, 1 MB, 10 MB and
//! 50 MB (the sizes the specification asks to test):
//!
//! ```sh
//! cargo run --release -p nashir-engine --example bench
//! ```
//!
//! Prints, per size: decode time, sectioning time, sections, first-section
//! build time, an Arabic search, and the engine's memory per source byte.

use std::time::Instant;

use nashir_engine::{load_bytes, search_document, Document};

fn generate(target: usize) -> String {
    let block = "## قسم تجريبي عن الأداء\n\nهذه فقرة عربية تحتوي على **نص عريض** و`code` ورابط [ناشر](https://example.com) وكلمة أحمد للبحث. \
English text is mixed in the same paragraph to exercise bidi detection.\n\n\
- عنصر أول\n- عنصر ثانٍ مع [^n]\n  - عنصر متداخل\n\n\
| العمود | Value | ملاحظة |\n|---|---:|---|\n| أ | 1 | نص |\n| ب | 2 | text |\n\n\
```rust\nfn main() {\n    println!(\"مرحبا\");\n}\n```\n\n\
```mermaid\nflowchart LR\n  A[البداية] --> B{قرار}\n  B -->|نعم| C[تم]\n```\n\n";
    let mut s = String::with_capacity(target + block.len());
    s.push_str("# مستند اختبار الأداء\n\n");
    while s.len() < target {
        s.push_str(block);
    }
    s.push_str("\n[^n]: حاشية مشتركة.\n");
    s
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    println!("| Size | Decode | Sections | Count | 1st section | Search «احمد» | Hits |");
    println!("|---|---:|---:|---:|---:|---:|---:|");
    for (label, size) in [
        ("100 KB", 100 * 1024),
        ("1 MB", 1 << 20),
        ("10 MB", 10 << 20),
        ("50 MB", 50 << 20),
    ] {
        let text = generate(size);
        let bytes = text.into_bytes();

        let t = Instant::now();
        let loaded = load_bytes(&bytes).expect("decode");
        let decode = ms(t);

        let t = Instant::now();
        let doc = Document::parse(loaded.text);
        let parse = ms(t);

        let t = Instant::now();
        let first = doc.section_markdown(0);
        let first_ms = ms(t);
        assert!(!first.is_empty());

        let t = Instant::now();
        let result = search_document(&doc, "احمد", 1000);
        let search = ms(t);

        println!(
            "| {label} | {decode:.1} ms | {parse:.1} ms | {} | {first_ms:.2} ms | {search:.1} ms | {}{} |",
            doc.sections.len(),
            result.total,
            if result.truncated { "+" } else { "" }
        );
    }
}
