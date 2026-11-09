//! The output of the Mermaid engine: a *scene* of vector shapes (SVG path
//! data) and text labels, in points, with colors named by token.
//!
//! The interface draws shapes on an NSH `Canvas` (`Path2D` takes SVG path
//! data) and places labels as native `Text` views over it, so Arabic is
//! shaped by the platform's text engine (DirectWrite on Windows), stays
//! selectable and is read by screen readers. Color tokens are resolved by
//! the interface for the light or dark theme; [`super::svg`] resolves them
//! for export.

use serde::Serialize;

use crate::direction::{first_strong, Direction};

/// A filled and/or stroked path.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Shape {
    /// SVG path data, in scene coordinates.
    pub d: String,
    /// Fill color token, or none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<&'static str>,
    /// Stroke color token, or none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<&'static str>,
    #[serde(rename = "strokeWidth")]
    pub stroke_width: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub dash: bool,
}

/// A text label centred (or aligned) in a box.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Label {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    pub size: f32,
    pub weight: u16,
    pub color: &'static str,
    /// `center`, `start` or `end` (logical, follows the label's direction).
    pub align: &'static str,
    /// `rtl` or `ltr`, from the label's first strong character.
    pub dir: &'static str,
    /// A background drawn behind the text (edge labels), or none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<&'static str>,
}

/// A laid-out diagram.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Scene {
    pub kind: &'static str,
    pub width: f32,
    pub height: f32,
    pub shapes: Vec<Shape>,
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Scene {
    pub fn new(kind: &'static str) -> Self {
        Scene {
            kind,
            width: 0.0,
            height: 0.0,
            shapes: Vec::new(),
            labels: Vec::new(),
            title: None,
        }
    }

    pub fn path(
        &mut self,
        d: String,
        fill: Option<&'static str>,
        stroke: Option<&'static str>,
        width: f32,
        dash: bool,
    ) {
        self.shapes.push(Shape {
            d,
            fill,
            stroke,
            stroke_width: width,
            dash,
        });
    }

    pub fn label(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: &str,
        size: f32,
        weight: u16,
        color: &'static str,
    ) {
        self.labels.push(Label {
            x,
            y,
            w,
            h,
            text: text.to_owned(),
            size,
            weight,
            color,
            align: "center",
            dir: label_dir(text),
            bg: None,
        });
    }

    pub fn label_aligned(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: &str,
        size: f32,
        weight: u16,
        color: &'static str,
        align: &'static str,
    ) {
        self.label(x, y, w, h, text, size, weight, color);
        if let Some(l) = self.labels.last_mut() {
            l.align = align;
        }
    }

    /// Grows the scene to include every shape and label, plus `margin`, and
    /// moves everything so the top-left corner is at (margin, margin).
    pub fn fit(&mut self, margin: f32) {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for s in &self.shapes {
            for (x, y) in path_points(&s.d) {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        for l in &self.labels {
            min_x = min_x.min(l.x);
            min_y = min_y.min(l.y);
            max_x = max_x.max(l.x + l.w);
            max_y = max_y.max(l.y + l.h);
        }
        if min_x == f32::MAX {
            self.width = margin * 2.0;
            self.height = margin * 2.0;
            return;
        }
        let dx = margin - min_x;
        let dy = margin - min_y;
        if dx.abs() > 0.01 || dy.abs() > 0.01 {
            for s in &mut self.shapes {
                s.d = translate_path(&s.d, dx, dy);
            }
            for l in &mut self.labels {
                l.x += dx;
                l.y += dy;
            }
        }
        self.width = (max_x - min_x + margin * 2.0).ceil();
        self.height = (max_y - min_y + margin * 2.0).ceil();
    }
}

fn label_dir(text: &str) -> &'static str {
    match first_strong(text) {
        Some(Direction::Rtl) => "rtl",
        _ => "ltr",
    }
}

/// Formats a coordinate compactly.
pub fn n(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// A rectangle with rounded corners.
pub fn rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> String {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    if r <= 0.01 {
        return format!("M{} {}H{}V{}H{}Z", n(x), n(y), n(x + w), n(y + h), n(x));
    }
    format!(
        "M{} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}Z",
        n(x + r),
        n(y),
        n(x + w - r),
        n(r),
        n(r),
        n(x + w),
        n(y + r),
        n(y + h - r),
        n(r),
        n(r),
        n(x + w - r),
        n(y + h),
        n(x + r),
        n(r),
        n(r),
        n(x),
        n(y + h - r),
        n(y + r),
        n(r),
        n(r),
        n(x + r),
        n(y),
    )
}

/// A circle as two arcs.
pub fn circle(cx: f32, cy: f32, r: f32) -> String {
    format!(
        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}Z",
        n(cx - r),
        n(cy),
        n(r),
        n(r),
        n(cx + r),
        n(cy),
        n(r),
        n(r),
        n(cx - r),
        n(cy)
    )
}

/// A closed polygon.
pub fn polygon(points: &[(f32, f32)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(&format!("{} {}", n(*x), n(*y)));
    }
    d.push('Z');
    d
}

/// An open polyline.
pub fn polyline(points: &[(f32, f32)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(&format!("{} {}", n(*x), n(*y)));
    }
    d
}

/// A filled arrowhead with its tip at `(x, y)` pointing along `(dx, dy)`.
pub fn arrow_head(x: f32, y: f32, dx: f32, dy: f32, size: f32) -> String {
    let len = (dx * dx + dy * dy).sqrt().max(0.0001);
    let (ux, uy) = (dx / len, dy / len);
    let (px, py) = (-uy, ux);
    let bx = x - ux * size;
    let by = y - uy * size;
    polygon(&[
        (x, y),
        (bx + px * size * 0.45, by + py * size * 0.45),
        (bx - px * size * 0.45, by - py * size * 0.45),
    ])
}

/// A diamond (rhombus) marker with one corner at `(x, y)` pointing along `(dx, dy)`.
pub fn diamond_marker(x: f32, y: f32, dx: f32, dy: f32, size: f32) -> String {
    let len = (dx * dx + dy * dy).sqrt().max(0.0001);
    let (ux, uy) = (dx / len, dy / len);
    let (px, py) = (-uy, ux);
    let mx = x - ux * size;
    let my = y - uy * size;
    let bx = x - ux * size * 2.0;
    let by = y - uy * size * 2.0;
    polygon(&[
        (x, y),
        (mx + px * size * 0.5, my + py * size * 0.5),
        (bx, by),
        (mx - px * size * 0.5, my - py * size * 0.5),
    ])
}

/// The approximate width of `text` at `size` points, used for layout. The
/// interface lays the label out natively inside the box, so an estimate
/// that errs slightly wide is what matters.
pub fn text_width(text: &str, size: f32) -> f32 {
    let mut w = 0.0;
    for c in text.chars() {
        let f = match c as u32 {
            0x20 => 0.30,
            0x21..=0x2F | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => 0.38,
            0x30..=0x39 => 0.56,
            0x41..=0x5A => 0.66,
            0x61..=0x7A => 0.54,
            0x064B..=0x065F | 0x0670 => 0.0,
            0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFEFF => 0.52,
            0x0590..=0x05FF => 0.55,
            0x1100..=0x11FF | 0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF => 1.0,
            0x1F300..=0x1FAFF => 1.1,
            _ => 0.58,
        };
        w += f;
    }
    w * size
}

/// Splits a Mermaid label into lines (`<br>`, `<br/>`, `\n`) and removes
/// surrounding quotes and Markdown emphasis marks.
pub fn label_lines(raw: &str) -> Vec<String> {
    let mut s = raw.trim().to_owned();
    if s.len() >= 2 && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('`') && s.ends_with('`')))
    {
        s = s[1..s.len() - 1].to_owned();
    }
    let s = s
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n")
        .replace("<BR>", "\n")
        .replace("\\n", "\n")
        .replace("#quot;", "\"")
        .replace("#amp;", "&")
        .replace("#lt;", "<")
        .replace("#gt;", ">")
        .replace("**", "")
        .replace("__", "");
    s.split('\n').map(|l| l.trim().to_owned()).collect()
}

/// The size of a block of lines at `size` points.
pub fn block_size(lines: &[String], size: f32) -> (f32, f32) {
    let w = lines.iter().map(|l| text_width(l, size)).fold(0.0, f32::max);
    let h = lines.len().max(1) as f32 * size * LINE_HEIGHT;
    (w, h)
}

pub const LINE_HEIGHT: f32 = 1.4;
pub const FONT: f32 = 14.0;

/// Every coordinate pair in path data (for bounds), arcs and curves included.
pub fn path_points(d: &str) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut nums: Vec<f32> = Vec::new();
    let mut cmd = 'M';
    let mut cur = (0.0f32, 0.0f32);
    let flush = |cmd: char, nums: &mut Vec<f32>, cur: &mut (f32, f32), out: &mut Vec<(f32, f32)>| {
        match cmd {
            'M' | 'L' | 'T' => {
                for p in nums.chunks(2).filter(|c| c.len() == 2) {
                    *cur = (p[0], p[1]);
                    out.push(*cur);
                }
            }
            'H' => {
                for &x in nums.iter() {
                    cur.0 = x;
                    out.push(*cur);
                }
            }
            'V' => {
                for &y in nums.iter() {
                    cur.1 = y;
                    out.push(*cur);
                }
            }
            'C' | 'S' | 'Q' => {
                for p in nums.chunks(2).filter(|c| c.len() == 2) {
                    out.push((p[0], p[1]));
                }
                if nums.len() >= 2 {
                    *cur = (nums[nums.len() - 2], nums[nums.len() - 1]);
                }
            }
            'A' => {
                for a in nums.chunks(7).filter(|c| c.len() == 7) {
                    let (rx, ry) = (a[0], a[1]);
                    let end = (a[5], a[6]);
                    // Bound the arc generously by its radii around both ends.
                    out.push((cur.0 - rx, cur.1 - ry));
                    out.push((cur.0 + rx, cur.1 + ry));
                    out.push((end.0 - rx, end.1 - ry));
                    out.push((end.0 + rx, end.1 + ry));
                    *cur = end;
                }
            }
            _ => {}
        }
        nums.clear();
    };
    let mut token = String::new();
    for c in d.chars() {
        if c.is_ascii_alphabetic() {
            if !token.is_empty() {
                nums.push(token.parse().unwrap_or(0.0));
                token.clear();
            }
            flush(cmd, &mut nums, &mut cur, &mut out);
            cmd = c;
        } else if c == ' ' || c == ',' {
            if !token.is_empty() {
                nums.push(token.parse().unwrap_or(0.0));
                token.clear();
            }
        } else if c == '-' && !token.is_empty() && !token.ends_with('e') {
            nums.push(token.parse().unwrap_or(0.0));
            token.clear();
            token.push(c);
        } else {
            token.push(c);
        }
    }
    if !token.is_empty() {
        nums.push(token.parse().unwrap_or(0.0));
    }
    flush(cmd, &mut nums, &mut cur, &mut out);
    out
}

/// Moves absolute path data by `(dx, dy)`.
pub fn translate_path(d: &str, dx: f32, dy: f32) -> String {
    let mut out = String::with_capacity(d.len());
    let mut cmd = 'M';
    let mut nums: Vec<f32> = Vec::new();
    let mut token = String::new();
    let emit = |cmd: char, nums: &mut Vec<f32>, out: &mut String| {
        let moved: Vec<f32> = match cmd {
            'H' => nums.iter().map(|x| x + dx).collect(),
            'V' => nums.iter().map(|y| y + dy).collect(),
            'A' => nums
                .chunks(7)
                .flat_map(|a| {
                    if a.len() == 7 {
                        vec![a[0], a[1], a[2], a[3], a[4], a[5] + dx, a[6] + dy]
                    } else {
                        a.to_vec()
                    }
                })
                .collect(),
            'Z' | 'z' => Vec::new(),
            _ => nums
                .iter()
                .enumerate()
                .map(|(i, v)| if i % 2 == 0 { v + dx } else { v + dy })
                .collect(),
        };
        out.push(cmd);
        let parts: Vec<String> = moved.iter().map(|v| n(*v)).collect();
        out.push_str(&parts.join(" "));
        nums.clear();
    };
    let mut started = false;
    for c in d.chars() {
        if c.is_ascii_alphabetic() {
            if !token.is_empty() {
                nums.push(token.parse().unwrap_or(0.0));
                token.clear();
            }
            if started {
                emit(cmd, &mut nums, &mut out);
            }
            started = true;
            cmd = c;
        } else if c == ' ' || c == ',' {
            if !token.is_empty() {
                nums.push(token.parse().unwrap_or(0.0));
                token.clear();
            }
        } else if c == '-' && !token.is_empty() && !token.ends_with('e') {
            nums.push(token.parse().unwrap_or(0.0));
            token.clear();
            token.push(c);
        } else {
            token.push(c);
        }
    }
    if !token.is_empty() {
        nums.push(token.parse().unwrap_or(0.0));
    }
    if started {
        emit(cmd, &mut nums, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_and_translate() {
        let d = rect(0.0, 0.0, 10.0, 10.0, 0.0);
        assert_eq!(d, "M0 0H10V10H0Z");
        assert_eq!(translate_path(&d, 5.0, 2.0), "M5 2H15V12H5Z");
        let pts = path_points(&d);
        assert!(pts.contains(&(10.0, 10.0)));
    }

    #[test]
    fn arc_translation_keeps_flags() {
        let d = circle(10.0, 10.0, 5.0);
        let t = translate_path(&d, 1.0, 1.0);
        assert_eq!(t, "M6 11A5 5 0 1 0 16 11A5 5 0 1 0 6 11Z");
    }

    #[test]
    fn labels() {
        assert_eq!(label_lines("\"سطر أول<br>second\""), vec!["سطر أول", "second"]);
        assert!(text_width("مرحبا", 14.0) > 0.0);
    }

    #[test]
    fn fit_moves_to_margin() {
        let mut s = Scene::new("x");
        s.path(rect(100.0, 50.0, 20.0, 10.0, 0.0), None, Some("edge"), 1.0, false);
        s.fit(8.0);
        assert_eq!(s.width, 36.0);
        assert_eq!(s.height, 26.0);
        assert!(s.shapes[0].d.starts_with("M8 8"));
    }
}
