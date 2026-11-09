# Markdown rendering

> **ملخص:** يُقسَّم المستند في Rust إلى أقسام، وتعرض قائمة NSH الافتراضية كل قسم بمكوّن TextView الأصلي، مع معالجة الحواشي والروابط والصور والاتجاه.

## Pipeline

1. **Load** (`nashir_engine::loader`) — validate and decode (see [Security & privacy](security-and-privacy.md)).
2. **Section** (`nashir_engine::document`) — split the source into sections:
   - every ATX (`#`) or setext (`===`/`---`) heading starts a section, so each outline entry maps to exactly one list row;
   - without headings, a section is cut at a blank line once it passes **24 KB**;
   - a code block larger than **96 KB** is closed at the end of one section and reopened (same fence and info string) at the start of the next, keeping each piece valid Markdown;
   - YAML front matter stays in the first section.
3. **Render** (`app/ui/Reader.tsx`) — an NSH `List` with one row per section; each row renders `Library.section(id, i)` with `TextView`.

## Document-wide definitions

Markdown footnote definitions (`[^1]: …`) and link reference definitions (`[ref]: url`) apply to the whole document, but a section is rendered alone. The engine therefore:

- removes footnote definitions from the section they were written in, and **appends** each definition to every section that references it, so footnotes appear under the section that cites them;
- appends the document's link reference definitions to sections that contain links (up to 64 KB of definitions).

## TextView configuration

```tsx
<TextView
  markdown={markdown()}                    // the section (highlighted when searching)
  selectable                               // select across paragraphs; Copy via menu or Ctrl+C
  options={{ frontmatter: first && hasFrontMatter }}
  extensions={[{ name: "mermaid", language: "mermaid" }]}
  renderCustom={…}                         // mermaid → MermaidView; frontmatter → metadata card
  renderImage={…}                          // local → data: URI; remote → opt-in
  codeBlockActions={[{ id: "copy", label: t("action.copy") }]}
  styles={readingStyles()}                 // per-part styles from the reading settings
  onLinkPress={({ url }) => …}
/>
```

`TextView` provides CommonMark plus GitHub's tables, task lists, strikethrough, autolinks, footnotes and alerts; math (`$…$`, `$$…$$`, typeset with KaTeX's TeX support); HTML inside Markdown (`<kbd>`, `<mark>`, `<details>`…); and syntax highlighting for ~200 languages.

## Typography

`app/ui/theme.ts` builds `readingStyles` from Settings: base size (default 17 pt), line height (1.8), font, heading scale (1.9 / 1.5 / 1.25 / 1.1 / 1.0 / 0.95), code font at 0.84–0.88×, and theme colors for links, code, quotes, tables, highlights. Content width is applied by the row (`narrow` 640, `medium` 760, `wide` 920, `full`).

## Direction

Each section carries its detected direction (one letter per section in `DocumentMeta.sectionDirections`). The row sets the view's `direction` (lists, quotes and tables lay out from that side) unless Settings forces RTL or LTR; paragraphs use `writingDirection: "auto"` and `textAlign: "auto"` so an English paragraph inside an Arabic document still reads left to right. Code blocks are always LTR. See [Localization & RTL](localization-and-rtl.md).

## Links

| Link | Behavior |
|---|---|
| `#anchor` | Scrolls to the heading's section (`Library.anchor` maps GitHub anchors, percent-encoded or not). |
| `#footnote-…` | Ignored (the footnote is in the same section). |
| `https:`, `http:`, `mailto:` | Opened outside with `openUrl` (allowed schemes only). |
| Relative `.md` path | Opened in a new tab, relative to the document's folder. |

## Images

- **Local** (relative, absolute or `file:`): read by `Library.image` on a worker thread and shown as a `data:` URI (NSH's `Image` does not load `file:` URLs). Over 20 MB or unreadable → an alt-text placeholder.
- **Remote**: not requested unless *Load images from the internet* is on; otherwise a placeholder names the host with a *Load image* button.
- Images keep their aspect ratio, up to their natural width and the content width.

## Search highlighting

When a search is active, rows whose section has matches render `Library.highlighted(id, i, query)`: matches are wrapped in `<mark>` (TextView's `highlight` part) except inside code, link targets, autolinks, HTML tags, heading markers and definitions, and never across a line break — so highlighting can't break the Markdown.
