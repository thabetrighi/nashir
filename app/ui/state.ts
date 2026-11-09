// Application state: settings, tabs, recent documents, bookmarks and the
// reading session, kept in signals and persisted locally with NSH's
// `Storage` (a JSON file in %APPDATA%\app.nashir.reader on Windows).
// Nothing leaves the device.

import { announce, batch, computed, effect, setColorScheme, signal, Storage } from "nsh";
import { lang, type Lang, t } from "./i18n";
import {
  closeDocument,
  type DocumentMeta,
  fileChanged,
  openDocument,
  type OpenError,
  openWelcome as openWelcomeNative,
} from "./native";

// ------------------------------------------------------------- settings

export type ThemeChoice = "system" | "light" | "sepia" | "dark";
export type WidthChoice = "narrow" | "medium" | "wide" | "full";
export type DirectionChoice = "auto" | "rtl" | "ltr";

export interface Settings {
  theme: ThemeChoice;
  fontSize: number;
  lineHeight: number;
  width: WidthChoice;
  font: string;
  codeFont: string;
  direction: DirectionChoice;
  remoteImages: boolean;
  restoreSession: boolean;
  maxFileMB: number;
  sidebar: boolean;
  sidebarTab: "contents" | "bookmarks";
  comfort: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  fontSize: 17,
  lineHeight: 1.8,
  width: "medium",
  font: "system",
  codeFont: "cascadia",
  direction: "auto",
  remoteImages: false,
  restoreSession: true,
  maxFileMB: 64,
  sidebar: true,
  sidebarTab: "contents",
  comfort: false,
};

export const FONT_SIZES = { min: 12, max: 30 };

export const settings = signal<Settings>({ ...DEFAULT_SETTINGS });

export function updateSettings(patch: Partial<Settings>) {
  settings.update((s) => ({ ...s, ...patch }));
}

export function zoomText(delta: number) {
  updateSettings({
    fontSize:
      delta === 0
        ? DEFAULT_SETTINGS.fontSize
        : Math.min(FONT_SIZES.max, Math.max(FONT_SIZES.min, settings().fontSize + delta)),
  });
}

// ------------------------------------------------------------- tabs

export type TabStatus = "loading" | "ready" | "error";

export interface Tab {
  key: string;
  path: string;
  name: string;
  status: TabStatus;
  meta: DocumentMeta | null;
  error: OpenError | null;
  pinned: boolean;
  /** The section at the top of the reader (restored on return). */
  section: number;
  /** The file changed on disk since it was opened. */
  changed: boolean;
  builtin: boolean;
}

export const tabs = signal<Tab[]>([]);
export const activeKey = signal<string | null>(null);
export const activeTab = computed<Tab | null>(() => tabs().find((x) => x.key === activeKey()) ?? null);

let keySeq = 0;
const closedStack: { path: string; builtin: boolean }[] = [];

function patchTab(key: string, patch: Partial<Tab>) {
  tabs.update((list) => list.map((x) => (x.key === key ? { ...x, ...patch } : x)));
}

/**
 * Where each tab is scrolled to. Kept outside the `tabs` signal on purpose:
 * scrolling changes it constantly, and nothing on screen depends on it
 * except the reader when it is (re)created.
 */
const scrollPositions = new Map<string, number>();

export function tabSection(key: string): number {
  return scrollPositions.get(key) ?? tabs().find((x) => x.key === key)?.section ?? 0;
}

export function setTabSection(key: string, section: number) {
  if (scrollPositions.get(key) === section) return;
  scrollPositions.set(key, section);
  const tab = tabs().find((x) => x.key === key);
  if (tab && !tab.builtin) {
    positions[tab.path] = section;
    schedulePersist();
  }
}

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function samePath(a: string, b: string): boolean {
  return a.replace(/\//g, "\\").toLowerCase() === b.replace(/\//g, "\\").toLowerCase();
}

/** Opens `paths` in tabs (or focuses them if already open). */
export async function openPaths(paths: string[], options: { focus?: boolean; pinned?: boolean } = {}) {
  let lastKey: string | null = null;
  for (const path of paths) {
    const existing = tabs().find((x) => !x.builtin && samePath(x.path, path));
    if (existing) {
      lastKey = existing.key;
      continue;
    }
    const key = `t${++keySeq}`;
    const tab: Tab = {
      key,
      path,
      name: baseName(path),
      status: "loading",
      meta: null,
      error: null,
      pinned: options.pinned ?? false,
      section: positions[path] ?? 0,
      changed: false,
      builtin: false,
    };
    tabs.update((list) => [...list, tab]);
    lastKey = key;
    load(key, path);
  }
  if (lastKey && options.focus !== false) activeKey.set(lastKey);
  schedulePersist();
}

async function load(key: string, path: string) {
  try {
    const meta = await openDocument(path, settings().maxFileMB);
    const tab = tabs().find((x) => x.key === key);
    if (!tab) {
      closeDocument(meta.id);
      return;
    }
    batch(() => {
      patchTab(key, { status: "ready", meta, error: null, name: meta.name, path: meta.path, changed: false });
    });
    remember(meta.path, meta.name);
    announce(t("a11y.opened", { name: meta.name }));
  } catch (e) {
    patchTab(key, { status: "error", error: e as OpenError });
  }
}

export async function openWelcome() {
  const existing = tabs().find((x) => x.builtin);
  if (existing) {
    activeKey.set(existing.key);
    return;
  }
  const key = `t${++keySeq}`;
  tabs.update((list) => [
    ...list,
    { key, path: "", name: t("action.welcome"), status: "loading", meta: null, error: null, pinned: false, section: 0, changed: false, builtin: true },
  ]);
  activeKey.set(key);
  try {
    const meta = await openWelcomeNative(lang());
    patchTab(key, { status: "ready", meta, name: meta.name });
  } catch (e) {
    patchTab(key, { status: "error", error: { code: "io", message: String(e) } });
  }
}

export function reloadTab(key: string) {
  const tab = tabs().find((x) => x.key === key);
  if (!tab || tab.builtin) return;
  if (tab.meta) closeDocument(tab.meta.id);
  patchTab(key, { status: "loading", meta: null, error: null, changed: false });
  load(key, tab.path);
}

export function closeTab(key: string, force = false) {
  const list = tabs();
  const index = list.findIndex((x) => x.key === key);
  if (index < 0) return;
  const tab = list[index];
  if (tab.pinned && !force) return;
  if (tab.meta) closeDocument(tab.meta.id);
  closedStack.push({ path: tab.path, builtin: tab.builtin });
  const rest = list.filter((x) => x.key !== key);
  batch(() => {
    tabs.set(rest);
    if (activeKey() === key) {
      const next = rest[Math.min(index, rest.length - 1)];
      activeKey.set(next ? next.key : null);
    }
  });
  schedulePersist();
}

export function closeOtherTabs(key: string) {
  for (const tab of tabs()) if (tab.key !== key && !tab.pinned) closeTab(tab.key);
}

export function reopenClosed() {
  const last = closedStack.pop();
  if (!last) return;
  if (last.builtin) openWelcome();
  else openPaths([last.path]);
}

export function togglePin(key: string) {
  tabs.update((list) => {
    const updated = list.map((x) => (x.key === key ? { ...x, pinned: !x.pinned } : x));
    // Pinned tabs come first, in their order.
    return [...updated.filter((x) => x.pinned), ...updated.filter((x) => !x.pinned)];
  });
  schedulePersist();
}

export function cycleTab(step: number) {
  const list = tabs();
  if (list.length < 2) return;
  const index = list.findIndex((x) => x.key === activeKey());
  activeKey.set(list[(index + step + list.length) % list.length].key);
}

export function dismissChange(key: string) {
  patchTab(key, { changed: false });
}

// ------------------------------------------------------------- change detection

let watcher: ReturnType<typeof setInterval> | null = null;

export function startWatching() {
  if (watcher) return;
  watcher = setInterval(() => {
    for (const tab of tabs()) {
      if (tab.status === "ready" && tab.meta && !tab.changed && !tab.builtin && fileChanged(tab.meta.id)) {
        patchTab(tab.key, { changed: true });
      }
    }
  }, 2000);
}

// ------------------------------------------------------------- recent & bookmarks

export interface RecentEntry {
  path: string;
  name: string;
  opened: number;
}

export interface Bookmark {
  anchor: string;
  text: string;
  level: number;
}

export const recent = signal<RecentEntry[]>([]);
export const bookmarks = signal<Record<string, Bookmark[]>>({});
let positions: Record<string, number> = {};

function remember(path: string, name: string) {
  recent.update((list) => [{ path, name, opened: Date.now() }, ...list.filter((r) => !samePath(r.path, path))].slice(0, 20));
  schedulePersist();
}

export function clearRecent() {
  recent.set([]);
  schedulePersist();
}

export function forgetRecent(path: string) {
  recent.update((list) => list.filter((r) => !samePath(r.path, path)));
  schedulePersist();
}

export function bookmarksFor(path: string): Bookmark[] {
  return bookmarks()[path] ?? [];
}

export function toggleBookmark(path: string, mark: Bookmark) {
  bookmarks.update((all) => {
    const list = all[path] ?? [];
    const exists = list.some((b) => b.anchor === mark.anchor);
    return { ...all, [path]: exists ? list.filter((b) => b.anchor !== mark.anchor) : [...list, mark] };
  });
  schedulePersist();
}

// ------------------------------------------------------------- toasts

export interface Toast {
  id: number;
  text: string;
  tone: "info" | "error";
}

export const toasts = signal<Toast[]>([]);
let toastSeq = 0;

export function toast(text: string, tone: Toast["tone"] = "info") {
  const id = ++toastSeq;
  toasts.update((list) => [...list.slice(-2), { id, text, tone }]);
  announce(text);
  setTimeout(() => toasts.update((list) => list.filter((x) => x.id !== id)), 3200);
}

// ------------------------------------------------------------- UI state

export const searchOpen = signal(false);
export const settingsOpen = signal(false);
export const fullscreen = signal(false);

// ------------------------------------------------------------- persistence

const KEYS = { settings: "settings.v1", lang: "lang.v1", recent: "recent.v1", session: "session.v1", bookmarks: "bookmarks.v1", positions: "positions.v1" };

let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function schedulePersist() {
  if (!loaded) return;
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(persist, 400);
}

function persist() {
  persistTimer = null;
  const session = {
    tabs: tabs()
      .filter((x) => !x.builtin)
      .map((x) => ({ path: x.path, pinned: x.pinned })),
    active: tabs().find((x) => x.key === activeKey())?.path ?? null,
  };
  const writes: [string, unknown][] = [
    [KEYS.settings, settings()],
    [KEYS.lang, lang()],
    [KEYS.recent, recent()],
    [KEYS.session, session],
    [KEYS.bookmarks, bookmarks()],
    [KEYS.positions, trimPositions()],
  ];
  for (const [key, value] of writes) {
    Storage.setItem(key, JSON.stringify(value)).catch(() => {
      // Over the 5 MB quota: drop reading positions, which regrow.
      if (key === KEYS.positions) positions = {};
    });
  }
}

function trimPositions(): Record<string, number> {
  const keep = new Set(recent().map((r) => r.path));
  const out: Record<string, number> = {};
  for (const path in positions) if (keep.has(path)) out[path] = positions[path];
  return out;
}

async function read<T>(key: string, fallback: T): Promise<T> {
  try {
    const raw = await Storage.getItem(key);
    return raw ? ({ ...(fallback as object), ...JSON.parse(raw) } as T) : fallback;
  } catch {
    return fallback;
  }
}

async function readArray<T>(key: string): Promise<T[]> {
  try {
    const raw = await Storage.getItem(key);
    const value = raw ? JSON.parse(raw) : [];
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

/** Loads persisted state; returns the session's documents to reopen. */
export async function loadPersisted(): Promise<{ tabs: { path: string; pinned: boolean }[]; active: string | null }> {
  const s = await read<Settings>(KEYS.settings, { ...DEFAULT_SETTINGS });
  settings.set(s);
  try {
    const savedLang = JSON.parse((await Storage.getItem(KEYS.lang)) ?? "null");
    if (savedLang === "ar" || savedLang === "en" || savedLang === "fr") lang.set(savedLang as Lang);
  } catch {
    // keep the default
  }
  recent.set(await readArray<RecentEntry>(KEYS.recent));
  bookmarks.set(await read<Record<string, Bookmark[]>>(KEYS.bookmarks, {}));
  positions = await read<Record<string, number>>(KEYS.positions, {});
  const session = await read<{ tabs: { path: string; pinned: boolean }[]; active: string | null }>(KEYS.session, { tabs: [], active: null });
  loaded = true;
  return s.restoreSession ? session : { tabs: [], active: null };
}

// Persist whenever settings or the language change, and follow the theme.
effect(() => {
  settings();
  lang();
  schedulePersist();
});

effect(() => {
  const theme = settings().theme;
  setColorScheme(theme === "dark" ? "dark" : theme === "system" ? "system" : "light");
});

export function focusPath(path: string) {
  const tab = tabs().find((x) => !x.builtin && samePath(x.path, path));
  if (tab) activeKey.set(tab.key);
}
