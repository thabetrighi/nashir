// Nashir (ناشر): a native Markdown reader built on Native Shell.
//
// The entry module: the window, its layout, the application menu and
// keyboard shortcuts, file drops, and start-up (file association arguments
// and the restored session).

import { computed, effect, keymap, mount, platform, signal, Text, View } from "nsh";
import { isAvailable, Menu, Window } from "nsh/desktop";
import { bookmarkCurrent, openDialog, stepHeading } from "./ui/actions";
import { EmptyState } from "./ui/EmptyState";
import { LANGUAGES, lang, t, uiRtl } from "./ui/i18n";
import { launchPaths, markdownPaths } from "./ui/native";
import { ErrorView, LoadingView, Reader } from "./ui/Reader";
import { SearchBar } from "./ui/SearchBar";
import { SettingsDialog } from "./ui/SettingsDialog";
import { Sidebar } from "./ui/Sidebar";
import {
  activeKey,
  activeTab,
  clearRecent,
  closeTab,
  cycleTab,
  focusPath,
  loadPersisted,
  openPaths,
  openWelcome,
  recent,
  reloadTab,
  reopenClosed,
  searchOpen,
  settings,
  settingsOpen,
  startWatching,
  type ThemeChoice,
  toast,
  toasts,
  updateSettings,
  zoomText,
} from "./ui/state";
import { StatusBar } from "./ui/StatusBar";
import { palette } from "./ui/theme";
import { TITLE_BAR_HEIGHT, TitleBar } from "./ui/TitleBar";

const dropping = signal(false);

function Toasts() {
  const p = palette;
  return (
    <View style={{ position: "absolute", bottom: 44, start: 0, end: 0, alignItems: "center", gap: 8, pointerEvents: "box-none" }}>
      <Each of={toasts()} key={(x) => x.id}>
        {(item) => (
          <View
            role="alert"
            style={{
              paddingHorizontal: 16,
              paddingVertical: 10,
              borderRadius: 10,
              maxWidth: 520,
              backgroundColor: item().tone === "error" ? p().danger : p().dark ? "#E6E8EB" : "#1C2127",
              boxShadow: [{ x: 0, y: 6, blur: 18, color: p().shadow }],
            }}
          >
            <Text style={{ color: item().tone === "error" ? "#FFFFFF" : p().dark ? "#111316" : "#FFFFFF", fontSize: 13.5 }}>{item().text}</Text>
          </View>
        )}
      </Each>
    </View>
  );
}

/** The main area: the active document, or the welcome screen. */
function Main() {
  // Re-create the reader when the tab, its status or its document change.
  const viewKey = computed(() => {
    const tab = activeTab();
    return tab ? `${tab.key}:${tab.status}:${tab.meta?.id ?? ""}` : "empty";
  });
  return (
    <View style={{ flex: 1 }}>
      <Each of={[viewKey()]} key={(k: string) => k}>
        {() => {
          const tab = activeTab();
          if (!tab) return <EmptyState dropping={dropping()} />;
          if (tab.status === "loading") return <LoadingView />;
          if (tab.status === "error") return <ErrorView tab={activeTab()!} />;
          return <Reader tab={activeTab()!} />;
        }}
      </Each>
      {/* Keyed by document: the bar lives as long as the search on that document. */}
      <Each of={searchOpen() && activeTab()?.meta ? [activeTab()!.meta!.id] : []} key={(id: number) => id}>
        {() => <SearchBar meta={activeTab()!.meta!} />}
      </Each>
    </View>
  );
}

function App() {
  const p = palette;
  const showSidebar = () => settings().sidebar && !settings().comfort && !!activeTab()?.meta;
  return (
    <View
      style={{ flex: 1, backgroundColor: p().chrome, direction: uiRtl() ? "rtl" : "ltr" }}
      onFileDragEnter={() => dropping.set(true)}
      onFileDragLeave={() => dropping.set(false)}
      onFileDrop={(e) => {
        dropping.set(false);
        const paths = markdownPaths(e.uris);
        const skipped = e.uris.length - paths.length;
        if (paths.length) openPaths(paths);
        if (skipped > 0) toast(t("error.notMarkdown", { n: skipped }), "error");
      }}
    >
      <TitleBar />
      <View style={{ flex: 1, flexDirection: "row" }}>
        {showSidebar() ? (
          <Each of={[`${activeKey()}:${activeTab()?.meta?.id}`]} key={(k: string) => k}>
            {() => <Sidebar tab={activeTab()!} meta={activeTab()!.meta!} />}
          </Each>
        ) : null}
        <Main />
      </View>
      {settings().comfort ? null : <StatusBar />}
      {dropping() && activeTab() ? (
        <View
          style={{ pointerEvents: "none", position: "absolute", top: TITLE_BAR_HEIGHT, start: 0, end: 0, bottom: 0, backgroundColor: p().overlay, alignItems: "center", justifyContent: "center" }}
        >
          <View style={{ paddingHorizontal: 28, paddingVertical: 18, borderRadius: 14, backgroundColor: p().page, borderWidth: 2, borderStyle: "dashed", borderColor: p().accent }}>
            <Text style={{ fontSize: 16, fontWeight: "600", color: p().accent }}>{t("empty.drop")}</Text>
          </View>
        </View>
      ) : null}
      <Toasts />
      <SettingsDialog />
    </View>
  );
}

// ------------------------------------------------------------- menu & keys

function themeItem(value: ThemeChoice, label: string) {
  return { label, checked: settings().theme === value, onSelect: () => updateSettings({ theme: value }) };
}

effect(() => {
  if (!isAvailable) return;
  const mac = platform.os === "macos";
  const tab = activeTab();
  Menu.setApplicationMenu([
    {
      label: t("menu.file"),
      submenu: [
        { label: `${t("action.open")}…`, shortcut: "mod+o", onSelect: openDialog },
        {
          label: t("menu.recent"),
          submenu: [
            ...recent()
              .slice(0, 10)
              .map((r) => ({ label: r.name, onSelect: () => openPaths([r.path]) })),
            { type: "separator" as const },
            { label: t("menu.clearRecent"), disabled: recent().length === 0, onSelect: clearRecent },
          ],
        },
        { label: t("action.reopenClosed"), shortcut: "mod+shift+t", onSelect: reopenClosed },
        { label: t("action.reload"), shortcut: "mod+shift+r", disabled: !tab || tab.builtin, onSelect: () => tab && reloadTab(tab.key) },
        { type: "separator" as const },
        { label: t("action.closeTab"), shortcut: "mod+w", disabled: !tab, onSelect: () => tab && closeTab(tab.key, true) },
        !mac && { type: "separator" as const },
        { role: "quit" as const, label: t("action.quit") },
      ],
    },
    {
      label: t("menu.edit"),
      submenu: [
        { role: "copy" as const },
        { role: "selectAll" as const },
        { type: "separator" as const },
        { label: t("action.search"), shortcut: "mod+f", disabled: !tab?.meta, onSelect: () => searchOpen.set(true) },
      ],
    },
    {
      label: t("menu.view"),
      submenu: [
        { label: t("action.sidebar"), shortcut: "mod+b", checked: settings().sidebar, onSelect: () => updateSettings({ sidebar: !settings().sidebar }) },
        { label: t("action.comfort"), shortcut: "mod+shift+m", checked: settings().comfort, onSelect: () => updateSettings({ comfort: !settings().comfort }) },
        { type: "separator" as const },
        { label: t("action.zoomIn"), shortcut: "mod+plus", onSelect: () => zoomText(1) },
        { label: t("action.zoomOut"), shortcut: "mod+minus", onSelect: () => zoomText(-1) },
        { label: t("action.zoomReset"), shortcut: "mod+0", onSelect: () => zoomText(0) },
        { type: "separator" as const },
        {
          label: t("settings.theme"),
          submenu: [
            themeItem("system", t("settings.theme.system")),
            themeItem("light", t("settings.theme.light")),
            themeItem("sepia", t("settings.theme.sepia")),
            themeItem("dark", t("settings.theme.dark")),
          ],
        },
        {
          label: t("settings.language"),
          submenu: LANGUAGES.map((l) => ({ label: l.name, checked: lang() === l.code, onSelect: () => lang.set(l.code) })),
        },
        { role: "toggleFullScreen" as const, label: t("action.fullscreen") },
        { type: "separator" as const },
        { label: `${t("action.settings")}…`, shortcut: "mod+,", onSelect: () => settingsOpen.set(true) },
      ],
    },
    {
      label: t("menu.go"),
      submenu: [
        { label: t("action.nextTab"), shortcut: "ctrl+tab", onSelect: () => cycleTab(1) },
        { label: t("action.prevTab"), shortcut: "ctrl+shift+tab", onSelect: () => cycleTab(-1) },
        { type: "separator" as const },
        { label: t("action.nextHeading"), shortcut: "alt+down", disabled: !tab?.meta, onSelect: () => stepHeading(1) },
        { label: t("action.prevHeading"), shortcut: "alt+up", disabled: !tab?.meta, onSelect: () => stepHeading(-1) },
        { label: t("action.bookmark"), shortcut: "mod+d", disabled: !tab?.meta || tab.builtin, onSelect: bookmarkCurrent },
      ],
    },
    {
      label: t("menu.help"),
      submenu: [
        { label: t("action.welcome"), shortcut: "f1", onSelect: openWelcome },
        { label: t("settings.about"), onSelect: () => settingsOpen.set(true) },
      ],
    },
  ]);
});

// Keyboard shortcuts. The keymap is the single place they run: NSH gives a
// keymap binding precedence over a menu item's shortcut of the same keys,
// so the menu above only displays them.
keymap.bind({
  "mod+o": () => openDialog(),
  "mod+f": () => activeTab()?.meta && searchOpen.set(true),
  "mod+b": () => updateSettings({ sidebar: !settings().sidebar }),
  "mod+shift+m": () => updateSettings({ comfort: !settings().comfort }),
  "mod+plus": () => zoomText(1),
  "mod+minus": () => zoomText(-1),
  "mod+0": () => zoomText(0),
  "mod+w": () => {
    const tab = activeTab();
    if (tab) closeTab(tab.key, true);
  },
  "mod+shift+t": () => reopenClosed(),
  "mod+shift+r": () => {
    const tab = activeTab();
    if (tab && !tab.builtin) reloadTab(tab.key);
  },
  "ctrl+tab": () => cycleTab(1),
  "ctrl+shift+tab": () => cycleTab(-1),
  "alt+down": () => stepHeading(1),
  "alt+up": () => stepHeading(-1),
  "mod+d": () => bookmarkCurrent(),
  "mod+,": () => settingsOpen.set(true),
  f1: () => openWelcome(),
});

// The window: our own title bar, and the document's name as the title.
if (isAvailable) Window.setTitleBar({ style: "hidden", height: TITLE_BAR_HEIGHT });
effect(() => {
  if (!isAvailable) return;
  const tab = activeTab();
  Window.setTitle(tab ? `${tab.name} — ${t("app.name")}` : t("app.name"));
});

// ------------------------------------------------------------- start-up

async function start() {
  const session = await loadPersisted();
  const launched = launchPaths();
  if (session.tabs.length) {
    const pinned = session.tabs.filter((x) => x.pinned).map((x) => x.path);
    const rest = session.tabs.filter((x) => !x.pinned).map((x) => x.path);
    if (pinned.length) await openPaths(pinned, { focus: false, pinned: true });
    if (rest.length) await openPaths(rest, { focus: false });
    if (session.active) focusPath(session.active);
  }
  if (launched.length) {
    await openPaths(launched);
  } else if (!session.tabs.length && recent().length === 0) {
    // First run: show the guided tour.
    await openWelcome();
  }
  startWatching();
}

start();
mount(App);
