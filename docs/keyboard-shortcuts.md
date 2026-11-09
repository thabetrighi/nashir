# Keyboard shortcuts

> **ملخص:** كل الاختصارات معرّفة في مكان واحد (`keymap` في App.tsx) وتظهر أيضاً في قائمة التطبيق.

All shortcuts are bound once with NSH's `keymap` in `app/App.tsx` (which takes precedence over menu shortcuts of the same keys); the application menu shows them. On macOS, `Ctrl` is `⌘`.

## Documents

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Open document(s) |
| <kbd>Ctrl</kbd>+<kbd>W</kbd> | Close current document |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>T</kbd> | Reopen last closed document |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>R</kbd> | Reload document from disk |
| <kbd>Ctrl</kbd>+<kbd>Tab</kbd> | Next document |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Tab</kbd> | Previous document |

## Reading & navigation

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Search in document |
| <kbd>Enter</kbd> / <kbd>F3</kbd> / <kbd>Ctrl</kbd>+<kbd>G</kbd> | Next result (while searching) |
| <kbd>Shift</kbd>+<kbd>F3</kbd> / <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>G</kbd> | Previous result |
| <kbd>Esc</kbd> | Close search, dialog or pop-up |
| <kbd>Alt</kbd>+<kbd>↓</kbd> / <kbd>Alt</kbd>+<kbd>↑</kbd> | Next / previous heading |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Bookmark the current heading |
| <kbd>↑</kbd> <kbd>↓</kbd> <kbd>PgUp</kbd> <kbd>PgDn</kbd> <kbd>Space</kbd> <kbd>Home</kbd> <kbd>End</kbd> | Scroll (native) |
| <kbd>Ctrl</kbd>+<kbd>A</kbd>, <kbd>Ctrl</kbd>+<kbd>C</kbd> | Select all / copy selected text (native) |

## View

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>+</kbd> | Larger text |
| <kbd>Ctrl</kbd>+<kbd>-</kbd> | Smaller text |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Reset text size |
| <kbd>Ctrl</kbd>+<kbd>B</kbd> | Show / hide sidebar |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>M</kbd> | Comfort reading mode |
| <kbd>F11</kbd> | Full screen (handled by NSH) |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Settings |
| <kbd>F1</kbd> | Welcome guide |
| <kbd>Alt</kbd>+<kbd>F4</kbd> | Exit (system) |

## Notes

- Shortcuts work with any keyboard layout (NSH matches the physical key a US layout would type), including Arabic layouts.
- `F11`, `Alt+F4` and `Ctrl+Q` are reserved by NSH and the system.
- Adding a shortcut: add it to the `keymap.bind({...})` call and, for discoverability, to the matching menu item's `shortcut` (display only), then to this page and to `empty.shortcuts` if it is essential.
