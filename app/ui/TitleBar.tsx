// The window's own title bar: the mark, document tabs and actions, in a
// draggable bar that leaves room for the system's window controls.

import { Scroll, Text, View } from "nsh";
import { Menu, titleBar } from "nsh/desktop";
import { IconButton } from "./controls";
import { Icon } from "./Icon";
import { t } from "./i18n";
import { Logo } from "./Logo";
import { copyPath, openDialog } from "./actions";
import {
  activeKey,
  closeOtherTabs,
  closeTab,
  reloadTab,
  searchOpen,
  settings,
  settingsOpen,
  type Tab,
  tabs,
  togglePin,
  updateSettings,
} from "./state";
import { palette } from "./theme";
import { revealInFolder } from "./native";

export const TITLE_BAR_HEIGHT = 44;

function tabMenu(tab: Tab) {
  return [
    { label: tab.pinned ? t("action.unpin") : t("action.pin"), onSelect: () => togglePin(tab.key) },
    !tab.builtin && { label: t("action.reload"), onSelect: () => reloadTab(tab.key) },
    { type: "separator" as const },
    !tab.builtin && { label: t("action.copyPath"), onSelect: () => copyPath(tab.path) },
    !tab.builtin && { label: t("action.reveal"), onSelect: () => revealInFolder(tab.path).catch(() => undefined) },
    { type: "separator" as const },
    { label: t("action.closeOthers"), onSelect: () => closeOtherTabs(tab.key) },
    { label: t("action.closeTab"), onSelect: () => closeTab(tab.key, true) },
  ];
}

function TabItem(props: { tab: () => Tab }) {
  const p = palette;
  const active = () => activeKey() === props.tab().key;
  return (
    <View
      role="tab"
      accessibilityLabel={t("a11y.tab", { name: props.tab().name })}
      accessibilityState={{ selected: active() }}
      onPress={() => activeKey.set(props.tab().key)}
      onContextMenu={(e) => Menu.popup(tabMenu(props.tab()), e.pagePosition)}
      style={{
        flexDirection: "row",
        alignItems: "center",
        gap: 7,
        height: 32,
        maxWidth: 220,
        minWidth: props.tab().pinned ? 44 : 120,
        paddingStart: 10,
        paddingEnd: 4,
        borderRadius: 9,
        backgroundColor: active() ? p().page : "transparent",
        borderWidth: active() ? 1 : 0,
        borderColor: p().border,
        cursor: "pointer",
      }}
      hoverStyle={{ backgroundColor: active() ? p().page : p().hover }}
    >
      {props.tab().status === "loading" ? (
        <View style={{ width: 8, height: 8, borderRadius: 4, backgroundColor: p().accent, opacity: 0.7, marginHorizontal: 3 }} />
      ) : props.tab().status === "error" ? (
        <Icon name="alert" size={14} color={p().danger} />
      ) : props.tab().builtin ? (
        <Icon name="book" size={14} color={active() ? p().accent : p().textMuted} />
      ) : (
        <Icon name="file" size={14} color={active() ? p().accent : p().textMuted} />
      )}
      {props.tab().pinned ? null : (
        <Text
          style={{
            flex: 1,
            fontSize: 13,
            fontWeight: active() ? "600" : "400",
            color: active() ? p().text : p().textSoft,
            numberOfLines: 1,
            textOverflow: "ellipsis-middle",
            writingDirection: "auto",
          }}
        >
          {props.tab().name}
        </Text>
      )}
      {props.tab().changed ? <View style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: p().warningText }} /> : null}
      {props.tab().pinned ? (
        <Icon name="pin" size={13} color={p().textMuted} />
      ) : (
        <View
          role="button"
          accessibilityLabel={t("action.closeTab")}
          onPress={() => closeTab(props.tab().key)}
          style={{ width: 22, height: 22, borderRadius: 6, alignItems: "center", justifyContent: "center", opacity: active() ? 1 : 0.55, cursor: "pointer" }}
          hoverStyle={{ backgroundColor: p().pressed, opacity: 1 }}
        >
          <Icon name="close" size={12} color={p().textSoft} strokeWidth={2} />
        </View>
      )}
    </View>
  );
}

export function TitleBar() {
  const p = palette;
  const bar = () => titleBar();
  return (
    <View
      desktop={{ windowDrag: true }}
      style={{
        height: bar().height || TITLE_BAR_HEIGHT,
        paddingLeft: bar().left + 10,
        paddingRight: bar().right + 8,
        flexDirection: "row",
        alignItems: "center",
        gap: 8,
        backgroundColor: p().chrome,
        borderBottomWidth: 1,
        borderColor: p().divider,
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", gap: 8, paddingHorizontal: 4 }}>
        <Logo size={22} tile={p().accent} ink={p().dark ? "#06201D" : "#FFFFFF"} />
        {tabs().length === 0 ? <Text style={{ fontSize: 14, fontWeight: "700", color: p().text }}>{t("app.name")}</Text> : null}
      </View>
      <Scroll horizontal scrollbar={false} style={{ flex: 1 }} contentStyle={{ flexDirection: "row", alignItems: "center", gap: 4 }}>
        <Each of={tabs()} key={(x: Tab) => x.key}>
          {(tab) => <TabItem tab={tab} />}
        </Each>
        <IconButton icon="plus" label={t("action.open")} size={16} onPress={openDialog} />
      </Scroll>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 2 }}>
        <IconButton icon="search" label={`${t("action.search")} (Ctrl+F)`} active={searchOpen()} disabled={!tabs().length} onPress={() => searchOpen.set(!searchOpen())} />
        <IconButton icon="sidebar" label={`${t("action.sidebar")} (Ctrl+B)`} active={settings().sidebar} onPress={() => updateSettings({ sidebar: !settings().sidebar })} />
        <IconButton
          icon={p().dark ? "sun" : "moon"}
          label={t("settings.theme")}
          onPress={() => updateSettings({ theme: p().dark ? "light" : "dark" })}
        />
        <IconButton icon="settings" label={`${t("action.settings")} (Ctrl+,)`} onPress={() => settingsOpen.set(true)} />
      </View>
    </View>
  );
}
