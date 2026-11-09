//! `erDiagram`.

use super::graph::{Edge, Graph, Marker, Shape};
use super::layout::Dir;
use super::scene::{label_lines, Scene};
use super::{statements_keep_braces, MermaidError};

pub fn render(src: &str) -> Result<Scene, MermaidError> {
    Ok(parse(src)?.render("er"))
}

fn cardinality(part: &str) -> Option<&'static str> {
    Some(match part {
        "||" => "1",
        "|o" | "o|" => "0..1",
        "}|" | "|{" => "1..*",
        "}o" | "o{" => "0..*",
        _ => return None,
    })
}

pub fn parse(src: &str) -> Result<Graph, MermaidError> {
    let mut g = Graph::new(Dir::TopBottom);
    let mut attrs: Vec<Vec<String>> = Vec::new();
    let mut open: Option<usize> = None;
    let mut lines = statements_keep_braces(src);
    let _header = lines.next();

    let ensure = |g: &mut Graph, attrs: &mut Vec<Vec<String>>, raw: &str| -> usize {
        let (id, alias) = match (raw.find('['), raw.rfind(']')) {
            (Some(a), Some(b)) if b > a => (raw[..a].trim(), Some(raw[a + 1..b].trim().trim_matches('"'))),
            _ => (raw.trim().trim_matches('"'), None),
        };
        let fresh = g.find(id).is_none();
        let n = g.node(id);
        if fresh || alias.is_some() {
            g.nodes[n].lines = vec![alias.unwrap_or(id).to_owned()];
            g.nodes[n].shape = Shape::Compartments;
        }
        while attrs.len() < g.nodes.len() {
            attrs.push(Vec::new());
        }
        n
    };

    for line in lines {
        let s = line.trim();
        if s.is_empty() || s.starts_with("%%") || s.starts_with("style ") || s.starts_with("classDef ") {
            continue;
        }
        if let Some(d) = s.strip_prefix("direction ") {
            g.dir = Dir::parse(d);
            continue;
        }
        if let Some(e) = open {
            if s == "}" {
                open = None;
            } else {
                // type name [PK, FK] ["comment"]
                let (main, comment) = match s.find('"') {
                    Some(q) => (s[..q].trim(), Some(s[q..].trim().trim_matches('"'))),
                    None => (s, None),
                };
                let mut row = main.split_whitespace().collect::<Vec<_>>().join(" ");
                if let Some(c) = comment {
                    row.push_str(" — ");
                    row.push_str(c);
                }
                attrs[e].push(row);
            }
            continue;
        }
        if let Some(name) = s.strip_suffix('{') {
            let n = ensure(&mut g, &mut attrs, name.trim());
            open = Some(n);
            continue;
        }
        // A ||--o{ B : label
        let (rel, label) = match s.split_once(':') {
            Some((r, l)) => (r.trim(), Some(l.trim().trim_matches('"'))),
            None => (s, None),
        };
        let parts: Vec<&str> = rel.split_whitespace().collect();
        if parts.len() == 3 {
            let op = parts[1];
            let (dashed, sep) = if op.contains("..") {
                (true, "..")
            } else {
                (false, "--")
            };
            if let Some((l, r)) = op.split_once(sep) {
                if let (Some(lc), Some(rc)) = (cardinality(l), cardinality(r)) {
                    let a = ensure(&mut g, &mut attrs, parts[0]);
                    let b = ensure(&mut g, &mut attrs, parts[2]);
                    let mut e = Edge::new(a, b);
                    e.end = Marker::None;
                    e.dashed = dashed;
                    e.start_text = Some(lc.to_owned());
                    e.end_text = Some(rc.to_owned());
                    if let Some(l) = label.filter(|l| !l.is_empty()) {
                        e.label = label_lines(l);
                    }
                    g.edges.push(e);
                    continue;
                }
            }
        }
        if parts.len() == 1 {
            ensure(&mut g, &mut attrs, parts[0]);
        }
    }
    if g.nodes.is_empty() {
        return Err(MermaidError::Empty);
    }
    for (node, a) in g.nodes.iter_mut().zip(attrs) {
        node.compartments = if a.is_empty() { Vec::new() } else { vec![a] };
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_and_relations() {
        let g = parse("erDiagram\n  CUSTOMER ||--o{ ORDER : places\n  ORDER ||--|{ LINE-ITEM : \"يحتوي\"\n  CUSTOMER {\n    string name PK \"الاسم\"\n    int age\n  }\n").unwrap();
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.edges[0].start_text.as_deref(), Some("1"));
        assert_eq!(g.edges[0].end_text.as_deref(), Some("0..*"));
        assert_eq!(g.edges[1].label, vec!["يحتوي"]);
        let c = g.find("CUSTOMER").unwrap();
        assert_eq!(
            g.nodes[c].compartments[0],
            vec!["string name PK — الاسم", "int age"]
        );
        assert!(render("erDiagram\nA ||--|| B : x").is_ok());
    }
}
