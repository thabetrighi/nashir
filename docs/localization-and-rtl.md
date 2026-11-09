# Localization & RTL

> **ملخص:** لغة الواجهة (العربية افتراضياً، مع الإنجليزية والفرنسية) مستقلة تماماً عن اتجاه المستند، الذي يُكتشف تلقائياً لكل قسم، ويمكن فرضه من الإعدادات.

Nashir keeps two independent things apart:

| | Decided by | Affects |
|---|---|---|
| **Interface language & direction** | Settings → Interface language (default: Arabic unless the system is English or French) | Menus, buttons, dialogs, title bar order, sidebar side |
| **Document direction** | Detected per section, or forced in Settings → Document direction | Paragraphs, lists, quotes, tables of the document |

So an English interface can read an Arabic document right-to-left, and vice versa.

## Interface languages

Strings live in `app/i18n/<lang>.json` (flat keys such as `"search.results": "{current} من {total}"`), separate from the components. `app/ui/i18n.ts` exposes:

- `lang` — a signal (`"ar" | "en" | "fr"`), persisted;
- `t(key, vars?)` — reactive lookup with `{name}` placeholders, falling back to English, then to the key;
- `uiRtl` — whether the interface lays out right to left; the root view sets `direction` from it, so rows, paddings with `start`/`end`, and the sidebar mirror automatically (`left`/`right` keep their sides, as in CSS);
- `formatNumber` / `formatBytes` — `Intl` formatting; Arabic uses Western digits (`ar-u-nu-latn`), common in technical writing and the Maghreb.

The application menu and window title are rebuilt in an `effect`, so changing the language applies everywhere at once, without a restart.

### Adding a language

1. Copy `app/i18n/en.json` to `app/i18n/<code>.json` and translate the values (keep the `{placeholders}`).
2. Import it in `app/ui/i18n.ts`, add it to `TABLES` and `LANGUAGES` (with `rtl: true` for right-to-left scripts such as Persian, Urdu or Hebrew), and widen the `Lang` type.
3. Add a `welcome/<code>.md` tour in `app/native/welcome/` and a branch in `Library::open_welcome` (optional; English is used otherwise).
4. Add `[CustomMessages]` and a `[Languages]` entry in `packaging/windows/nashir.iss` if Inno Setup ships that language.

Missing keys fall back to English, so a partial translation is safe to ship.

## Document direction

### Detection

`nashir_engine::direction` counts *strong* characters, skipping fenced code, inline code, URLs, link targets and HTML tags. RTL scripts are Arabic, Hebrew, Syriac, Thaana, N'Ko and their presentation forms; Arabic-Indic digits and diacritics are neutral. A section (and the document) is **RTL when ≥ 30 % of its strong characters are RTL** — the threshold accounts for Arabic technical writing full of Latin terms. This runs once per section during parsing.

### Rendering

- The section's container gets `direction: "rtl" | "ltr"`, so list markers, quote bars and table columns start on the correct side.
- Paragraph text uses `writingDirection: "auto"` and `textAlign: "auto"`: each paragraph's base direction follows its first strong character (Unicode bidi rules P2–P3), so mixed documents read naturally.
- Code blocks force `direction: "ltr"`, `writingDirection: "ltr"` and left alignment. Inline code, paths (`C:\Users\…`) and numbers inside Arabic text are handled by the platform's bidi algorithm (DirectWrite), so punctuation is not mirrored incorrectly.
- Mermaid labels carry their own direction (first strong character) and are laid out by the system's text engine, so Arabic in diagrams is shaped and ordered correctly.

### Overrides

Settings → Document direction: *Automatic* (default), *Right to left* or *Left to right* — applies to every section of every document.

## Fonts

On Windows, Segoe UI covers Arabic, Latin and mixed text well; Tahoma, Arial, Sakkal Majalla, Traditional Arabic and Georgia are offered for reading, Cascadia Mono / Consolas / Courier New for code. Missing fonts fall back to the system's.

## Search in Arabic

See [Testing & QA](testing-and-qa.md) for the cases covered: diacritics (tashkeel) and tatweel are ignored; أ إ آ ٱ match ا; ى matches ي; ة matches ه; ؤ matches و; ئ matches ي; Arabic-Indic digits match ASCII digits; Latin is case-insensitive. Matches map back to the original text, so highlights cover the word exactly as written.
