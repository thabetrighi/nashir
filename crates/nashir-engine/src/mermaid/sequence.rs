//! `sequenceDiagram`.

use std::collections::HashMap;

use super::scene::*;
use super::{statements, MermaidError};

struct Participant {
    id: String,
    label: Vec<String>,
    actor: bool,
}

enum Item {
    Message {
        from: usize,
        to: usize,
        text: Vec<String>,
        dashed: bool,
        head: Head,
        number: Option<usize>,
    },
    Note {
        from: usize,
        to: usize,
        side: NoteSide,
        text: Vec<String>,
    },
    Open {
        kind: String,
        text: String,
    },
    Else {
        text: String,
    },
    Close,
}

#[derive(Clone, Copy)]
enum Head {
    Arrow,
    Open,
    Cross,
    Async,
}

#[derive(Clone, Copy, PartialEq)]
enum NoteSide {
    Left,
    Right,
    Over,
}

const ARROWS: &[(&str, bool, Head)] = &[
    ("-->>", true, Head::Arrow),
    ("->>", false, Head::Arrow),
    ("--x", true, Head::Cross),
    ("-x", false, Head::Cross),
    ("--)", true, Head::Async),
    ("-)", false, Head::Async),
    ("-->", true, Head::Open),
    ("->", false, Head::Open),
];

pub fn render(src: &str) -> Result<Scene, MermaidError> {
    let mut parts: Vec<Participant> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut items: Vec<Item> = Vec::new();
    let mut autonumber = false;
    let mut counter = 0;
    let mut title = None;

    let mut get = |parts: &mut Vec<Participant>, id: &str| -> usize {
        let id = id.trim().trim_start_matches(['+', '-']).trim();
        if let Some(&i) = index.get(id) {
            return i;
        }
        parts.push(Participant {
            id: id.to_owned(),
            label: vec![id.to_owned()],
            actor: false,
        });
        index.insert(id.to_owned(), parts.len() - 1);
        parts.len() - 1
    };

    let mut lines = statements(src);
    let _header = lines.next();
    for line in lines {
        let s = line.trim();
        if s.is_empty()
            || s.starts_with("%%")
            || s.starts_with("activate ")
            || s.starts_with("deactivate ")
            || s.starts_with("link ")
            || s.starts_with("links ")
            || s.starts_with("box")
            || s.starts_with("create ")
            || s.starts_with("destroy ")
        {
            if s.starts_with("box") {
                // `box` groups are drawn as plain participants.
            }
            continue;
        }
        if s == "autonumber" || s.starts_with("autonumber ") {
            autonumber = true;
            continue;
        }
        if let Some(t) = s.strip_prefix("title") {
            title = Some(t.trim_start_matches(':').trim().to_owned());
            continue;
        }
        for (kw, actor) in [("participant ", false), ("actor ", true)] {
            if let Some(rest) = s.strip_prefix(kw) {
                let (id, label) = match rest.split_once(" as ") {
                    Some((id, label)) => (id.trim(), label.trim()),
                    None => (rest.trim(), rest.trim()),
                };
                let i = get(&mut parts, id);
                parts[i].label = label_lines(label);
                parts[i].actor = actor;
            }
        }
        if s.starts_with("participant ") || s.starts_with("actor ") {
            continue;
        }
        let lower = s.to_lowercase();
        if lower.starts_with("note ") {
            let rest = &s[5..];
            let (place, text) = match rest.split_once(':') {
                Some((p, t)) => (p.trim(), t.trim()),
                None => continue,
            };
            let pl = place.to_lowercase();
            let (side, who) = if let Some(w) = pl.strip_prefix("left of ") {
                (NoteSide::Left, &place[place.len() - w.len()..])
            } else if let Some(w) = pl.strip_prefix("right of ") {
                (NoteSide::Right, &place[place.len() - w.len()..])
            } else if let Some(w) = pl.strip_prefix("over ") {
                (NoteSide::Over, &place[place.len() - w.len()..])
            } else {
                continue;
            };
            let (a, b) = match who.split_once(',') {
                Some((a, b)) => (get(&mut parts, a), get(&mut parts, b)),
                None => {
                    let a = get(&mut parts, who);
                    (a, a)
                }
            };
            items.push(Item::Note {
                from: a.min(b),
                to: a.max(b),
                side,
                text: label_lines(text),
            });
            continue;
        }
        let first = lower.split_whitespace().next().unwrap_or("");
        if matches!(
            first,
            "loop" | "alt" | "opt" | "par" | "critical" | "break" | "rect"
        ) {
            let text = s[first.len()..].trim().to_owned();
            items.push(Item::Open {
                kind: first.to_owned(),
                text: if first == "rect" { String::new() } else { text },
            });
            continue;
        }
        if matches!(first, "else" | "and" | "option") {
            items.push(Item::Else {
                text: s[first.len()..].trim().to_owned(),
            });
            continue;
        }
        if s == "end" {
            items.push(Item::Close);
            continue;
        }
        // A message.
        let found = ARROWS
            .iter()
            .filter_map(|a| s.find(a.0).map(|p| (p, a)))
            .min_by_key(|(p, a)| (*p, std::cmp::Reverse(a.0.len())));
        if let Some((pos, (tok, dashed, head))) = found {
            let from = &s[..pos];
            let rest = &s[pos + tok.len()..];
            let (to, text) = match rest.split_once(':') {
                Some((t, x)) => (t, x.trim()),
                None => (rest, ""),
            };
            let a = get(&mut parts, from);
            let b = get(&mut parts, to);
            let number = if autonumber {
                counter += 1;
                Some(counter)
            } else {
                None
            };
            items.push(Item::Message {
                from: a,
                to: b,
                text: label_lines(text),
                dashed: *dashed,
                head: *head,
                number,
            });
        }
    }
    if parts.is_empty() {
        return Err(MermaidError::Empty);
    }

    // Column widths.
    let n = parts.len();
    let mut box_w: Vec<f32> = parts
        .iter()
        .map(|p| (block_size(&p.label, FONT).0 + 28.0).max(96.0))
        .collect();
    let any_actor = parts.iter().any(|p| p.actor);
    let box_h: f32 = parts
        .iter()
        .map(|p| block_size(&p.label, FONT).1 + 20.0)
        .fold(if any_actor { 54.0 } else { 42.0 }, f32::max);
    let mut gaps = vec![60.0f32; n.saturating_sub(1)];
    for item in &items {
        if let Item::Message {
            from,
            to,
            text,
            number,
            ..
        } = item
        {
            let w = block_size(text, 13.0).0 + if number.is_some() { 28.0 } else { 0.0 } + 36.0;
            let (a, b) = ((*from).min(*to), (*from).max(*to));
            if a == b {
                if a + 1 < n {
                    gaps[a] = gaps[a].max(w * 0.6 + 40.0);
                }
            } else {
                let span = (b - a) as f32;
                for g in gaps.iter_mut().take(b).skip(a) {
                    *g = g.max(w / span - 40.0);
                }
            }
        }
        if let Item::Note { from, to, side, text } = item {
            let w = block_size(text, 13.0).0 + 24.0;
            if *side == NoteSide::Right && *to + 1 < n {
                gaps[*to] = gaps[*to].max(w + 20.0);
            }
            if *side == NoteSide::Left && *from > 0 {
                gaps[*from - 1] = gaps[*from - 1].max(w + 20.0);
            }
        }
    }
    for w in box_w.iter_mut() {
        *w = w.max(96.0);
    }
    let mut centers = Vec::with_capacity(n);
    let mut x = 0.0;
    for i in 0..n {
        if i > 0 {
            x += box_w[i - 1] / 2.0 + gaps[i - 1] + box_w[i] / 2.0;
        }
        centers.push(x);
    }

    let mut scene = Scene::new("sequence");
    scene.title = title.clone();
    let top = if let Some(t) = &title {
        scene.label(
            centers[0] - box_w[0] / 2.0,
            -36.0,
            centers[n - 1] - centers[0] + box_w[0],
            24.0,
            t,
            16.0,
            700,
            "text",
        );
        0.0
    } else {
        0.0
    };
    let mut y = top + box_h + 24.0;

    // Body: messages, notes and frames.
    let mut body: Vec<Shape> = Vec::new();
    let mut frames: Vec<(String, String, f32, Vec<f32>)> = Vec::new(); // kind, text, start y, else ys
    let mut frame_spans: Vec<(usize, usize)> = Vec::new();
    let mut pending_labels: Vec<Label> = Vec::new();
    let mut scratch = Scene::new("sequence");
    for item in &items {
        match item {
            Item::Message {
                from,
                to,
                text,
                dashed,
                head,
                number,
            } => {
                let (_, th) = block_size(text, 13.0);
                let label_y = y;
                y += th.max(16.0) + 6.0;
                let x1 = centers[*from];
                let x2 = centers[*to];
                let t = if text.is_empty() {
                    String::new()
                } else {
                    text.join("\n")
                };
                let t = match number {
                    Some(k) => format!("{k}. {t}"),
                    None => t,
                };
                if from == to {
                    let w = 36.0;
                    let d = format!(
                        "M{} {}H{}V{}H{}",
                        n_(x1),
                        n_(y),
                        n_(x1 + w),
                        n_(y + 24.0),
                        n_(x1 + 4.0)
                    );
                    scratch.path(d, None, Some("edge"), 1.5, *dashed);
                    head_at(&mut scratch, *head, x1 + 2.0, y + 24.0, -1.0, 0.0);
                    if !t.is_empty() {
                        scratch.label_aligned(
                            x1 + w + 6.0,
                            label_y,
                            block_size(std::slice::from_ref(&t), 13.0).0 + 10.0,
                            th.max(16.0),
                            &t,
                            13.0,
                            400,
                            "text",
                            "start",
                        );
                    }
                    y += 34.0;
                } else {
                    let dir = if x2 > x1 { 1.0 } else { -1.0 };
                    scratch.path(
                        format!("M{} {}H{}", n_(x1), n_(y), n_(x2 - dir * 2.0)),
                        None,
                        Some("edge"),
                        1.5,
                        *dashed,
                    );
                    head_at(&mut scratch, *head, x2, y, dir, 0.0);
                    if !t.is_empty() {
                        let w = (x2 - x1).abs();
                        scratch.label(x1.min(x2), label_y, w, th.max(16.0), &t, 13.0, 400, "text");
                    }
                    y += 14.0;
                }
                if let Some(span) = frame_spans.last_mut() {
                    span.0 = span.0.min((*from).min(*to));
                    span.1 = span.1.max((*from).max(*to));
                }
            }
            Item::Note { from, to, side, text } => {
                let (tw, th) = block_size(text, 13.0);
                let w = tw + 20.0;
                let h = th + 12.0;
                let x = match side {
                    NoteSide::Left => centers[*from] - 12.0 - w,
                    NoteSide::Right => centers[*to] + 12.0,
                    NoteSide::Over => {
                        let a = centers[*from] - 30.0;
                        let b = centers[*to] + 30.0;
                        let span = b - a;
                        if span > w {
                            a
                        } else {
                            (centers[*from] + centers[*to]) / 2.0 - w / 2.0
                        }
                    }
                };
                let w = if *side == NoteSide::Over {
                    w.max(centers[*to] - centers[*from] + 60.0)
                } else {
                    w
                };
                scratch.path(
                    rect(x, y, w, h, 3.0),
                    Some("note"),
                    Some("noteBorder"),
                    1.0,
                    false,
                );
                scratch.label(
                    x + 6.0,
                    y + 6.0,
                    w - 12.0,
                    th,
                    &text.join("\n"),
                    13.0,
                    400,
                    "text",
                );
                y += h + 14.0;
                if let Some(span) = frame_spans.last_mut() {
                    span.0 = span.0.min(*from);
                    span.1 = span.1.max(*to);
                }
            }
            Item::Open { kind, text } => {
                frames.push((kind.clone(), text.clone(), y, Vec::new()));
                frame_spans.push((usize::MAX, 0));
                y += 30.0;
            }
            Item::Else { text } => {
                if let Some(f) = frames.last_mut() {
                    f.3.push(y);
                    if !text.is_empty() {
                        f.1.push_str(&format!("\u{1F}{text}"));
                    }
                }
                y += 26.0;
            }
            Item::Close => {
                if let (Some((kind, text, start, elses)), Some(span)) = (frames.pop(), frame_spans.pop()) {
                    let (a, b) = if span.0 == usize::MAX { (0, n - 1) } else { span };
                    let depth = frames.len() as f32;
                    let x0 = centers[a] - box_w[a] / 2.0 + depth * 8.0 - 6.0;
                    let x1 = centers[b] + box_w[b] / 2.0 - depth * 8.0 + 6.0;
                    let h = y - start + 4.0;
                    body.push(Shape {
                        d: rect(x0, start, x1 - x0, h, 4.0),
                        fill: if kind == "rect" { Some("cluster") } else { None },
                        stroke: Some("clusterBorder"),
                        stroke_width: 1.2,
                        dash: false,
                    });
                    if kind != "rect" {
                        let tag = kind.clone();
                        let tw = text_width(&tag, 12.0) + 16.0;
                        body.push(Shape {
                            d: polygon(&[
                                (x0, start),
                                (x0 + tw, start),
                                (x0 + tw, start + 14.0),
                                (x0 + tw - 6.0, start + 20.0),
                                (x0, start + 20.0),
                            ]),
                            fill: Some("clusterBorder"),
                            stroke: None,
                            stroke_width: 0.0,
                            dash: false,
                        });
                        pending_labels.push(make_label(
                            x0,
                            start + 1.0,
                            tw,
                            18.0,
                            &tag,
                            12.0,
                            700,
                            "bg",
                            "center",
                        ));
                        let mut parts_text = text.split('\u{1F}');
                        if let Some(first) = parts_text.next().filter(|t| !t.is_empty()) {
                            pending_labels.push(make_label(
                                x0 + tw + 8.0,
                                start + 1.0,
                                x1 - x0 - tw - 16.0,
                                18.0,
                                &format!("[{first}]"),
                                12.5,
                                600,
                                "muted",
                                "start",
                            ));
                        }
                        for (k, ey) in elses.iter().enumerate() {
                            body.push(Shape {
                                d: format!("M{} {}H{}", n_(x0), n_(*ey), n_(x1)),
                                fill: None,
                                stroke: Some("clusterBorder"),
                                stroke_width: 1.0,
                                dash: true,
                            });
                            if let Some(t) = parts_text.next() {
                                let _ = k;
                                pending_labels.push(make_label(
                                    x0 + 8.0,
                                    ey + 3.0,
                                    x1 - x0 - 16.0,
                                    18.0,
                                    &format!("[{t}]"),
                                    12.5,
                                    600,
                                    "muted",
                                    "start",
                                ));
                            }
                        }
                    }
                    if let Some(parent) = frame_spans.last_mut() {
                        parent.0 = parent.0.min(a);
                        parent.1 = parent.1.max(b);
                    }
                    y += 12.0;
                }
            }
        }
    }
    let bottom = y + 10.0;

    // Lifelines, then frames, then messages, then participant boxes.
    for &cx in &centers {
        scene.path(
            format!("M{} {}V{}", n_(cx), n_(top + box_h), n_(bottom)),
            None,
            Some("clusterBorder"),
            1.2,
            true,
        );
    }
    scene.shapes.extend(body);
    scene.shapes.extend(scratch.shapes);
    scene.labels.extend(scratch.labels);
    scene.labels.extend(pending_labels);
    for (i, p) in parts.iter().enumerate() {
        for by in [top, bottom] {
            let bx = centers[i] - box_w[i] / 2.0;
            if p.actor {
                draw_actor(&mut scene, centers[i], by);
                scene.label(
                    bx,
                    by + 34.0,
                    box_w[i],
                    18.0,
                    &p.label.join(" "),
                    13.0,
                    600,
                    "text",
                );
            } else {
                scene.path(
                    rect(bx, by, box_w[i], box_h, 6.0),
                    Some("node"),
                    Some("nodeBorder"),
                    1.4,
                    false,
                );
                let (_, th) = block_size(&p.label, FONT);
                scene.label(
                    bx + 4.0,
                    by + (box_h - th) / 2.0,
                    box_w[i] - 8.0,
                    th,
                    &p.label.join("\n"),
                    FONT,
                    600,
                    "text",
                );
            }
        }
        let _ = &p.id;
    }
    scene.fit(12.0);
    Ok(scene)
}

fn n_(v: f32) -> String {
    n(v)
}

fn make_label(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    size: f32,
    weight: u16,
    color: &'static str,
    align: &'static str,
) -> Label {
    let mut s = Scene::new("x");
    s.label_aligned(x, y, w, h, text, size, weight, color, align);
    s.labels.pop().unwrap()
}

fn head_at(scene: &mut Scene, head: Head, x: f32, y: f32, dx: f32, dy: f32) {
    match head {
        Head::Arrow => scene.path(arrow_head(x, y, dx, dy, 10.0), Some("edge"), None, 0.0, false),
        Head::Open => {}
        Head::Async => {
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let (ux, uy) = (dx / len, dy / len);
            scene.path(
                format!(
                    "M{} {}L{} {}",
                    n(x - ux * 9.0 - uy * 6.0),
                    n(y - uy * 9.0 + ux * 6.0),
                    n(x),
                    n(y)
                ),
                None,
                Some("edge"),
                1.5,
                false,
            )
        }
        Head::Cross => {
            let s = 5.0;
            let cx = x - dx * 6.0;
            scene.path(
                format!(
                    "M{} {}L{} {}M{} {}L{} {}",
                    n(cx - s),
                    n(y - s),
                    n(cx + s),
                    n(y + s),
                    n(cx - s),
                    n(y + s),
                    n(cx + s),
                    n(y - s)
                ),
                None,
                Some("edge"),
                2.0,
                false,
            )
        }
    }
}

fn draw_actor(scene: &mut Scene, cx: f32, y: f32) {
    scene.path(
        circle(cx, y + 6.0, 5.5),
        Some("node"),
        Some("nodeBorder"),
        1.4,
        false,
    );
    scene.path(
        format!(
            "M{} {}V{}M{} {}H{}M{} {}L{} {}M{} {}L{} {}",
            n(cx),
            n(y + 11.5),
            n(y + 22.0),
            n(cx - 9.0),
            n(y + 15.0),
            n(cx + 9.0),
            n(cx),
            n(y + 22.0),
            n(cx - 7.0),
            n(y + 31.0),
            n(cx),
            n(y + 22.0),
            n(cx + 7.0),
            n(y + 31.0)
        ),
        None,
        Some("nodeBorder"),
        1.6,
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_notes_and_frames() {
        let s = render(
            "sequenceDiagram\n  autonumber\n  actor U as المستخدم\n  participant A as التطبيق\n  U->>A: افتح الملف\n  A-->>U: تم\n  loop كل ثانيتين\n    A->A: تحقق من التغييرات\n  end\n  alt تغير\n    A->>U: إشعار\n  else لم يتغير\n    A--)U: لا شيء\n  end\n  Note over U,A: ملاحظة\n",
        )
        .unwrap();
        assert!(s.width > 200.0);
        assert!(s.labels.iter().any(|l| l.text == "1. افتح الملف"));
        assert!(s.labels.iter().any(|l| l.text == "loop"));
        assert!(s.labels.iter().any(|l| l.text == "[لم يتغير]"));
        assert!(s.labels.iter().any(|l| l.text == "المستخدم" && l.dir == "rtl"));
    }
}
