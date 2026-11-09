//! SVG export of a [`Scene`], with color tokens resolved for the light or
//! dark theme. Text is exported as `<text>` elements (with their direction)
//! so the file stays editable and searchable in other tools.

use super::scene::{n, Scene, LINE_HEIGHT};

/// The color of a token in the light (`dark == false`) or dark theme. These
/// are the same values as the interface's `app/ui/theme.ts`.
pub fn color(token: &str, dark: bool) -> &'static str {
    let light = !dark;
    match token {
        "bg" => {
            if light {
                "#FFFFFF"
            } else {
                "#16191D"
            }
        }
        "node" => {
            if light {
                "#EAF5F3"
            } else {
                "#1C2A29"
            }
        }
        "nodeBorder" => {
            if light {
                "#0F766E"
            } else {
                "#2DD4BF"
            }
        }
        "edge" => {
            if light {
                "#475569"
            } else {
                "#94A3B8"
            }
        }
        "edgeLabel" => {
            if light {
                "#FFFFFF"
            } else {
                "#16191D"
            }
        }
        "text" => {
            if light {
                "#1F2937"
            } else {
                "#E5E7EB"
            }
        }
        "muted" => {
            if light {
                "#64748B"
            } else {
                "#94A3B8"
            }
        }
        "cluster" => {
            if light {
                "#F6F5F0"
            } else {
                "#1C2024"
            }
        }
        "clusterBorder" => {
            if light {
                "#CBD5E1"
            } else {
                "#3B4452"
            }
        }
        "note" => {
            if light {
                "#FEF3C7"
            } else {
                "#3A2F12"
            }
        }
        "noteBorder" => {
            if light {
                "#F59E0B"
            } else {
                "#D97706"
            }
        }
        "accent" => {
            if light {
                "#0F766E"
            } else {
                "#14B8A6"
            }
        }
        "danger" => {
            if light {
                "#DC2626"
            } else {
                "#F87171"
            }
        }
        "success" => {
            if light {
                "#16A34A"
            } else {
                "#22C55E"
            }
        }
        "onSeries" => "#FFFFFF",
        "s0" => {
            if light {
                "#0F766E"
            } else {
                "#0D9488"
            }
        }
        "s1" => {
            if light {
                "#2563EB"
            } else {
                "#3B82F6"
            }
        }
        "s2" => {
            if light {
                "#C2410C"
            } else {
                "#EA580C"
            }
        }
        "s3" => {
            if light {
                "#7C3AED"
            } else {
                "#8B5CF6"
            }
        }
        "s4" => {
            if light {
                "#BE185D"
            } else {
                "#DB2777"
            }
        }
        "s5" => {
            if light {
                "#0E7490"
            } else {
                "#0891B2"
            }
        }
        "s6" => {
            if light {
                "#4D7C0F"
            } else {
                "#65A30D"
            }
        }
        "s7" => {
            if light {
                "#A16207"
            } else {
                "#CA8A04"
            }
        }
        _ => {
            if light {
                "#1F2937"
            } else {
                "#E5E7EB"
            }
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The scene as a standalone SVG document.
pub fn to_svg(scene: &Scene, dark: bool) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" font-family=\"'Segoe UI', 'IBM Plex Sans Arabic', Tahoma, 'Noto Sans Arabic', sans-serif\">\n",
        w = n(scene.width),
        h = n(scene.height)
    ));
    if let Some(t) = &scene.title {
        out.push_str(&format!("<title>{}</title>\n", esc(t)));
    }
    out.push_str(&format!(
        "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n",
        color("bg", dark)
    ));
    for s in &scene.shapes {
        out.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{} stroke-linejoin=\"round\" stroke-linecap=\"round\"/>\n",
            s.d,
            s.fill.map(|f| color(f, dark)).unwrap_or("none"),
            s.stroke.map(|f| color(f, dark)).unwrap_or("none"),
            n(s.stroke_width),
            if s.dash { " stroke-dasharray=\"5 4\"" } else { "" }
        ));
    }
    for l in &scene.labels {
        if let Some(bg) = l.bg {
            out.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\"/>\n",
                n(l.x),
                n(l.y),
                n(l.w),
                n(l.h),
                color(bg, dark)
            ));
        }
        let lines: Vec<&str> = l.text.split('\n').collect();
        let lh = l.size * LINE_HEIGHT;
        let block = lh * lines.len() as f32;
        let top = l.y + (l.h - block) / 2.0;
        let rtl = l.dir == "rtl";
        let (anchor, x) = match (l.align, rtl) {
            ("center", _) => ("middle", l.x + l.w / 2.0),
            ("start", false) | ("end", true) => ("start", l.x),
            _ => ("end", l.x + l.w),
        };
        // With direction="rtl", "start" means the right edge; express the
        // anchor in absolute terms by flipping it.
        let anchor = if rtl && anchor != "middle" {
            if anchor == "start" {
                "end"
            } else {
                "start"
            }
        } else {
            anchor
        };
        out.push_str(&format!(
            "<text font-size=\"{}\" font-weight=\"{}\" fill=\"{}\" text-anchor=\"{}\"{}>",
            n(l.size),
            l.weight,
            color(l.color, dark),
            anchor,
            if rtl {
                " direction=\"rtl\" unicode-bidi=\"plaintext\""
            } else {
                ""
            }
        ));
        for (i, line) in lines.iter().enumerate() {
            let baseline = top + lh * i as f32 + lh / 2.0 + l.size * 0.35;
            out.push_str(&format!(
                "<tspan x=\"{}\" y=\"{}\">{}</tspan>",
                n(x),
                n(baseline),
                esc(line)
            ));
        }
        out.push_str("</text>\n");
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_valid_svg_with_rtl_text() {
        let scene = super::super::render("graph LR\nA[مرحبا] -->|نعم| B[World & <you>]").unwrap();
        let svg = to_svg(&scene, false);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("direction=\"rtl\""));
        assert!(svg.contains("World &amp; &lt;you&gt;"));
        assert!(svg.trim_end().ends_with("</svg>"));
        let dark = to_svg(&scene, true);
        assert!(dark.contains("#16191D"));
    }
}
