# Nashir · ناشر — Brand guide

**Nashir** (ناشر, *publisher*) is a calm, native Markdown reader that treats Arabic as a first-class language. The identity follows the product: quiet, precise, made for long reading.

## The mark

The mark is the Arabic letter **ن (noon)**, the first letter of ناشر:

- the **bowl** is drawn calligraphically — thicker at its base and tapered at its tips — and doubles as an open page;
- the **nuqta** is the rhombus dot of Arabic calligraphy (not a circle), the point of focus above the page;
- the **tile** is a rounded square (radius 12 on a 64 grid) in Nashir teal.

| File | Use |
|---|---|
| `logo-mark.svg` | The mark on its tile, 64×64 master. Use everywhere the app is represented. |
| `logo-mark-mono.svg` | The noon alone in `currentColor`, for one-color contexts (watermarks, monochrome UI). |
| `app-icon.svg` | The 1024×1024 application icon: tile with breathing room, sheen and soft shadow. |
| `wordmark.svg` | Mark + “Nashir” + “ناشر”, for headers and documentation. |
| `social-preview.svg/.png` | 1280×640 repository card. |
| `png/icon-*.png` | Rendered sizes 16–1024. 16–32 come from the flat mark (no shadow) for crispness. |
| `nashir.ico` | Windows icon, 16–256 (also in `packaging/windows/`). |

The in-app mark (`app/ui/Logo.tsx`) draws the same path data natively.

**Clear space:** keep at least ¼ of the tile's width free around the mark. **Minimum size:** 16 px. Do not rotate, outline, recolor the noon in anything but white or the ink of the current theme, or add effects.

## Color

Teal was chosen for calm focus and good contrast in both themes; paper tones support long reading.

| Token | Light | Dark | Use |
|---|---|---|---|
| Accent | `#0F766E` | `#2DD4BF` | Brand, links, selection, primary buttons |
| Accent strong | `#0B5E58` | `#5EEAD4` | Hover/pressed accent |
| Accent soft | `#E3F2EF` | `#163532` | Selected rows, soft fills |
| Page | `#FFFFFF` | `#191C20` | Reading surface |
| Chrome | `#F4F4F1` | `#111316` | Title bar, tabs, status bar |
| Panel | `#F8F8F6` | `#15181B` | Sidebar |
| Text | `#1C2127` | `#E6E8EB` | Body text |
| Text muted | `#6B7480` | `#8B939C` | Secondary text |
| Mark (search) | `#FDE68A` | `#7A5B00` | Search highlights |

**Paper** (sepia) reading theme: page `#F8F1E3`, text `#3B2F22`. Gradient for the tile: `#19C3AF → #0F8A80 → #0B5E58` (icon), `#14B8A6 → #0F766E` (mark).

Diagram palette (Mermaid): `s0 #0F766E · s1 #2563EB · s2 #C2410C · s3 #7C3AED · s4 #BE185D · s5 #0E7490 · s6 #4D7C0F · s7 #A16207` (light), with brighter dark variants. All values live in `app/ui/theme.ts` and `crates/nashir-engine/src/mermaid/svg.rs`.

## Typography

- **Interface and reading (Windows):** Segoe UI — excellent Arabic, Latin and mixed text through DirectWrite. Alternatives offered in Settings: Tahoma, Arial, Sakkal Majalla, Traditional Arabic, Georgia.
- **Code:** Cascadia Mono (Consolas, Courier New as alternatives).
- **Wordmark:** a bold sans (Segoe UI / IBM Plex Sans Arabic where available).

Reading defaults: 17 pt text, line height 1.8, 760 pt measure. Headings scale 1.9 / 1.5 / 1.25 / 1.1 / 1.0 / 0.95.

## Voice

Short, warm and direct. Arabic first; never translate word-for-word. Errors say what happened and what to do next (“The file wasn't found. It may have been moved or deleted.” + *Try again*). Privacy is stated plainly: everything stays on the device.

## Icons

The interface icons (`app/ui/Icon.tsx`) are original 24-point line icons: 1.8 pt strokes, round caps and joins, drawn natively on a Canvas from SVG path data.
