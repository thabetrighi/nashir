# Settings & storage

> **ملخص:** تُحفظ الإعدادات والملفات الأخيرة والإشارات والجلسة محلياً فقط عبر Storage في NSH، داخل ملف JSON في مجلد التطبيق، ولا يُحفظ أي محتوى من المستندات.

## Where

Nashir uses NSH's `Storage` (string keys → JSON strings, up to 5 MB). On Windows it is one file:

```
%APPDATA%\app.nashir.reader\storage.json
```

(`app.nashir.reader` is the `id` in `app/nsh.json`.) Linux: `$XDG_DATA_HOME/app.nashir.reader/`; macOS: `~/Library/Application Support/app.nashir.reader/`. The portable and installed builds share it. A damaged file is set aside as `storage.json.corrupt` by NSH and Nashir starts with defaults.

## What

| Key | Content |
|---|---|
| `settings.v1` | Theme, text size, line spacing, width, fonts, document direction, remote images, restore session, max file size, sidebar, comfort mode |
| `lang.v1` | Interface language |
| `recent.v1` | Up to 20 recent documents: path, name, time opened |
| `session.v1` | Open tabs (path, pinned) and the active one |
| `bookmarks.v1` | Per document path: bookmarked headings (anchor, text, level) |
| `positions.v1` | Per recent document: the section last read (trimmed to recent documents) |

**Never stored:** document contents, search queries, diagram sources, clipboard contents.

## Defaults

| Setting | Default | Range |
|---|---|---|
| Theme | System | System, Light, Paper, Dark |
| Text size | 17 pt | 12–30 (Ctrl + / Ctrl − / Ctrl 0) |
| Line spacing | 1.8 | 1.2–2.6 |
| Content width | Medium (760 pt) | Narrow 640, Medium 760, Wide 920, Full |
| Reading font | Segoe UI | Segoe UI, Tahoma, Arial, Sakkal Majalla, Traditional Arabic, Georgia |
| Code font | Cascadia Mono | Cascadia Mono, Consolas, Courier New |
| Document direction | Automatic | Automatic, RTL, LTR |
| Load remote images | Off | — |
| Restore last session | On | — |
| Maximum file size | 64 MB | 16, 64, 128, 256 MB |

## How it is written

`app/ui/state.ts` keeps state in signals and writes on change with a 400 ms debounce (`schedulePersist`). Scroll positions are tracked outside the reactive tab list (they change on every scroll) and saved with the rest. If a write ever exceeds the 5 MB quota, positions are dropped first (they regrow). Reads tolerate missing or malformed values and merge them over defaults, so adding a setting never breaks older files; keys are versioned (`.v1`) for future migrations.

## Resetting

Settings → *Restore defaults* resets the reading and appearance settings. To erase everything, close Nashir and delete the `%APPDATA%\app.nashir.reader` folder. Uninstalling keeps it (it is the user's data).
