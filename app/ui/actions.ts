// Commands shared by the menu, the keyboard and the interface.

import { Clipboard } from "nsh";
import { t } from "./i18n";
import { pickMarkdown } from "./native";
import { currentHeading, scrollToSection } from "./Reader";
import { activeTab, openPaths, toast, toggleBookmark } from "./state";

export async function openDialog() {
  try {
    const paths = await pickMarkdown(t("action.open"), "Markdown");
    if (paths.length) openPaths(paths);
  } catch (e) {
    toast(String(e), "error");
  }
}

export async function copyPath(path: string) {
  try {
    await Clipboard.writeText(path);
    toast(t("action.copied"));
  } catch {
    // clipboard unavailable
  }
}

/** Moves to the next (`+1`) or previous (`-1`) heading of the outline. */
export function stepHeading(dir: number) {
  const meta = activeTab()?.meta;
  if (!meta || !meta.outline.length) return;
  const i = currentHeading();
  const next = Math.max(0, Math.min(meta.outline.length - 1, i < 0 ? (dir > 0 ? 0 : -1) : i + dir));
  if (next >= 0) scrollToSection(meta.outline[next].section);
}

/** Bookmarks the heading under the top of the reader. */
export function bookmarkCurrent() {
  const tab = activeTab();
  const meta = tab?.meta;
  const i = currentHeading();
  if (!tab || tab.builtin || !meta || i < 0) return;
  const h = meta.outline[i];
  toggleBookmark(tab.path, { anchor: h.anchor, text: h.text, level: h.level });
}
