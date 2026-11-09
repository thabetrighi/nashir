//! Native Mermaid rendering.
//!
//! Mermaid's reference implementation is JavaScript that needs a browser
//! DOM to measure and draw. Nashir is a native app with no browser engine,
//! so this module re-implements the diagram types the specification asks
//! for in Rust: each source is parsed into a model, laid out, and turned
//! into a [`Scene`] of vector shapes and text labels that the interface
//! draws natively (see `docs/mermaid-rendering.md`).
//!
//! Supported: flowchart / graph, sequenceDiagram, classDiagram,
//! stateDiagram(-v2), erDiagram, gantt, pie, mindmap, gitGraph, journey.
//! Anything else returns [`MermaidError::Unsupported`] so the reader shows a
//! clear message and the source.

mod charts;
mod class;
mod er;
mod flowchart;
pub mod graph;
pub mod layout;
pub mod scene;
mod sequence;
mod state;
pub mod svg;
mod tree;

use std::fmt;

use serde::Serialize;

pub use scene::{Label, Scene, Shape};

/// Sources larger than this are refused (they would not be readable anyway).
pub const MAX_SOURCE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MermaidError {
    /// The diagram has no content.
    Empty,
    /// A diagram type Nashir does not draw yet (its keyword).
    Unsupported(String),
    /// The source is larger than [`MAX_SOURCE_BYTES`].
    TooLarge,
}

impl MermaidError {
    pub fn code(&self) -> &'static str {
        match self {
            MermaidError::Empty => "empty",
            MermaidError::Unsupported(_) => "unsupported",
            MermaidError::TooLarge => "too_large",
        }
    }
}

impl fmt::Display for MermaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MermaidError::Empty => write!(f, "empty diagram"),
            MermaidError::Unsupported(k) => write!(f, "unsupported diagram type: {k}"),
            MermaidError::TooLarge => write!(f, "diagram source too large"),
        }
    }
}

impl std::error::Error for MermaidError {}

/// The diagram types this engine draws, by keyword.
pub const SUPPORTED: &[&str] = &[
    "flowchart",
    "graph",
    "sequenceDiagram",
    "classDiagram",
    "stateDiagram",
    "erDiagram",
    "gantt",
    "pie",
    "mindmap",
    "gitGraph",
    "journey",
];

/// The diagram keyword of `src`: its first word after comments, front
/// matter and `%%{init}%%` directives.
pub fn diagram_keyword(src: &str) -> Option<String> {
    statements(src)
        .next()
        .and_then(|h| h.split_whitespace().next().map(str::to_owned))
}

/// Parses and lays out a Mermaid diagram.
pub fn render(src: &str) -> Result<Scene, MermaidError> {
    if src.len() > MAX_SOURCE_BYTES {
        return Err(MermaidError::TooLarge);
    }
    let keyword = diagram_keyword(src).ok_or(MermaidError::Empty)?;
    match keyword.as_str() {
        "graph" | "flowchart" | "flowchart-elk" => flowchart::render(src),
        "sequenceDiagram" => sequence::render(src),
        "classDiagram" | "classDiagram-v2" => class::render(src),
        "stateDiagram" | "stateDiagram-v2" => state::render(src),
        "erDiagram" => er::render(src),
        "gantt" => charts::gantt(src),
        "pie" => charts::pie(src),
        "journey" => charts::journey(src),
        "mindmap" => tree::mindmap(src),
        "gitGraph" | "gitGraph:" => tree::gitgraph(src),
        other => Err(MermaidError::Unsupported(other.trim_end_matches(':').to_owned())),
    }
}

#[derive(Serialize)]
struct JsonOk<'a> {
    ok: bool,
    scene: &'a Scene,
}

#[derive(Serialize)]
struct JsonErr<'a> {
    ok: bool,
    error: &'a str,
    detail: String,
}

/// [`render`] as JSON for the interface: `{"ok":true,"scene":{…}}` or
/// `{"ok":false,"error":"unsupported","detail":"quadrantChart"}`.
pub fn render_json(src: &str) -> String {
    match render(src) {
        Ok(scene) => serde_json::to_string(&JsonOk {
            ok: true,
            scene: &scene,
        })
        .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"internal\",\"detail\":\"\"}".into()),
        Err(e) => {
            let detail = match &e {
                MermaidError::Unsupported(k) => k.clone(),
                _ => String::new(),
            };
            serde_json::to_string(&JsonErr {
                ok: false,
                error: e.code(),
                detail,
            })
            .unwrap()
        }
    }
}

/// Mermaid lines, comments and directives removed, split on `;` outside
/// quotes and brackets (flowcharts and sequences allow `;` as a separator).
pub(crate) fn statements(src: &str) -> impl Iterator<Item = String> + '_ {
    split_statements(src, true).into_iter()
}

/// Like [`statements`] but without splitting on `;` (class, state and ER
/// bodies).
pub(crate) fn statements_keep_braces(src: &str) -> impl Iterator<Item = String> + '_ {
    split_statements(src, false).into_iter()
}

fn split_statements(src: &str, semicolons: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut lines = src.lines().peekable();
    // Front matter (`---` config block).
    while lines.peek().is_some_and(|l| l.trim().is_empty()) {
        lines.next();
    }
    if lines.peek().is_some_and(|l| l.trim() == "---") {
        lines.next();
        for l in lines.by_ref() {
            if l.trim() == "---" {
                break;
            }
        }
    }
    for line in lines {
        let t = line.trim();
        if t.is_empty() || t.starts_with("%%") {
            continue;
        }
        if !semicolons {
            out.push(line.trim_end().to_owned());
            continue;
        }
        let mut depth = 0i32;
        let mut quote = false;
        let mut cur = String::new();
        for c in line.chars() {
            match c {
                '"' => quote = !quote,
                '[' | '(' | '{' if !quote => depth += 1,
                ']' | ')' | '}' if !quote => depth -= 1,
                _ => {}
            }
            if c == ';' && !quote && depth <= 0 {
                if !cur.trim().is_empty() {
                    out.push(cur.trim_end().to_owned());
                }
                cur.clear();
            } else {
                cur.push(c);
            }
        }
        if !cur.trim().is_empty() {
            out.push(cur.trim_end().to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_type_renders() {
        let samples = [
            "flowchart TD\nA-->B",
            "sequenceDiagram\nA->>B: hi",
            "classDiagram\nA <|-- B",
            "stateDiagram-v2\n[*] --> A",
            "erDiagram\nA ||--o{ B : has",
            "gantt\nsection S\nTask :2026-01-01, 3d",
            "pie\n\"a\" : 1\n\"b\" : 2",
            "mindmap\n  root\n    child",
            "gitGraph\n  commit\n  commit",
            "journey\n  section S\n    Task: 3: Me",
        ];
        for s in samples {
            let scene = render(s).unwrap_or_else(|e| panic!("{s}: {e}"));
            assert!(scene.width > 0.0 && scene.height > 0.0, "{s}");
            assert!(!scene.shapes.is_empty(), "{s}");
        }
    }

    #[test]
    fn unsupported_and_comments() {
        assert_eq!(
            render("quadrantChart\n x").unwrap_err(),
            MermaidError::Unsupported("quadrantChart".into())
        );
        assert_eq!(
            diagram_keyword("%% comment\n%%{init: {}}%%\ngraph LR\nA-->B").as_deref(),
            Some("graph")
        );
        assert_eq!(
            diagram_keyword("---\ntitle: x\n---\nsequenceDiagram\nA->>B: x").as_deref(),
            Some("sequenceDiagram")
        );
        assert!(render("---\ntitle: x\n---\ngraph LR\nA-->B").is_ok());
        assert_eq!(render("").unwrap_err(), MermaidError::Empty);
        let json = render_json("sankey-beta\n");
        assert!(json.contains("\"unsupported\"") && json.contains("sankey-beta"));
        assert!(render_json("graph LR\nA-->B").starts_with("{\"ok\":true"));
    }
}
