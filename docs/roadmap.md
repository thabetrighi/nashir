# Roadmap & known limitations

> **ملخص:** ما الذي لم يُنجز بعد، وما القيود الحالية بصراحة، وما المخطط له بعد الإصدار الأول.

## Known limitations (0.1.0)

| Area | Limitation | Why / workaround |
|---|---|---|
| Keyboard navigation | <kbd>Tab</kbd> moves between text fields only; buttons are not reachable by Tab on Windows. Every action has a keyboard shortcut instead ([list](keyboard-shortcuts.md)). | NSH 0.4's Windows backend moves focus between text fields only. Revisit when NSH adds focus traversal for pressables. |
| Screen readers | Toasts are announced with `announce()`, but NSH 0.4 on Windows does not expose alert dialogs and toasts through UI Automation, and reports every heading as level 1. | NSH limitation; tested labels, roles and states are in place. |
| Single instance | Double-clicking a second `.md` file starts a second window instead of adding a tab to the first. | Needs an IPC channel between instances; planned. |
| Diagram export | SVG only; PNG export is not available yet. | NSH's native SVG renderer draws no `<text>`, so a PNG needs text rasterisation in Rust (e.g. resvg + system fonts). |
| Mermaid | Styling directives (`classDef`, `style`, `linkStyle`), icons, click handlers and the newest syntaxes are ignored; other diagram types (quadrant, sankey, timeline, C4…) show a message with the source. | See [Mermaid rendering](mermaid-rendering.md). |
| Math | Typeset by `TextView` with KaTeX's TeX subset; unsupported TeX shows as source. | NSH behavior. |
| Search | Matches inside one Markdown line; a phrase that spans formatting (`**a** b`) is matched only if typed with the markup. | Search runs on the source text for speed. |
| Very large documents | Sectioning a 50 MB file takes ≈ 1.5 s on a worker thread before the first section shows. | Streaming sectioning is planned. |
| Fonts | No bundled Arabic font; Segoe UI (Windows) is the default. | Bundling e.g. IBM Plex Sans Arabic (OFL) is planned. |

## Next (0.2)

- Single-instance with tabs from file association.
- PNG export of diagrams; copy diagram as image.
- Bundled reading fonts (IBM Plex Sans Arabic, Noto Naskh Arabic).
- Print and PDF export.
- Streaming load for files > 20 MB (show the first sections while the rest is parsed).
- Folder mode: a sidebar of a folder's Markdown files and search across them.

## Later

From the specification's future features: HTML export, MDX, more Mermaid types and styling, themes, plugins (NSH's signed TSX packages fit this well), optional sync, presentation mode — without compromising simplicity or performance.
