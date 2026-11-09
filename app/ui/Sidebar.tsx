// The sidebar: the document's outline (with the current section
// highlighted, filtering and bookmarks) and the saved bookmarks.

import { computed, List, type ListRef, signal, Text, View } from "nsh";
import { Input } from "nsh/base";
import { Icon } from "./Icon";
import { t } from "./i18n";
import type { DocumentMeta, OutlineItem } from "./native";
import { currentHeading, scrollToSection } from "./Reader";
import { type Bookmark, bookmarksFor, settings, type Tab, toggleBookmark, updateSettings } from "./state";
import { palette } from "./theme";

function OutlineRow(props: { item: () => OutlineItem; index: () => number; path: string; minLevel: number; marked: (anchor: string) => boolean }) {
  const p = palette;
  const active = () => currentHeading() === props.index();
  const hovering = signal(false);
  const marked = () => props.marked(props.item().anchor);
  return (
    <View
      role="link"
      accessibilityLabel={t("a11y.heading", { level: props.item().level, text: props.item().text })}
      accessibilityState={{ selected: active() }}
      onPress={() => scrollToSection(props.item().section)}
      onMouseEnter={() => hovering.set(true)}
      onMouseLeave={() => hovering.set(false)}
      style={{
        flexDirection: "row",
        alignItems: "center",
        minHeight: 30,
        marginHorizontal: 8,
        borderRadius: 7,
        paddingStart: 10 + (props.item().level - props.minLevel) * 14,
        paddingEnd: 4,
        backgroundColor: active() ? p().selected : "transparent",
        cursor: "pointer",
      }}
      hoverStyle={{ backgroundColor: active() ? p().selected : p().hover }}
    >
      <View style={{ width: 3, height: 16, borderRadius: 2, marginEnd: 8, backgroundColor: active() ? p().accent : "transparent" }} />
      <Text
        style={{
          flex: 1,
          fontSize: props.item().level <= 1 ? 13.5 : 13,
          fontWeight: props.item().level <= 2 ? "600" : "400",
          color: active() ? p().accent : props.item().level <= 2 ? p().text : p().textSoft,
          writingDirection: "auto",
          textAlign: "auto",
          numberOfLines: 2,
          paddingVertical: 5,
        }}
      >
        {props.item().text}
      </Text>
      <View
        role="button"
        accessibilityLabel={marked() ? t("action.removeBookmark") : t("action.bookmark")}
        onPress={() => toggleBookmark(props.path, { anchor: props.item().anchor, text: props.item().text, level: props.item().level })}
        style={{ width: 26, height: 26, alignItems: "center", justifyContent: "center", borderRadius: 6, opacity: marked() || hovering() ? 1 : 0, cursor: "pointer" }}
        hoverStyle={{ backgroundColor: p().pressed }}
      >
        <Icon name="star" size={14} color={marked() ? p().accent : p().textMuted} fill={marked() ? p().accent : undefined} />
      </View>
    </View>
  );
}

function SidebarTabButton(props: { label: string; selected: boolean; onPress: () => void }) {
  const p = palette;
  return (
    <View
      role="tab"
      accessibilityLabel={props.label}
      accessibilityState={{ selected: props.selected }}
      onPress={props.onPress}
      style={{ flex: 1, height: 30, borderRadius: 8, alignItems: "center", justifyContent: "center", backgroundColor: props.selected ? p().page : "transparent", borderWidth: props.selected ? 1 : 0, borderColor: p().border, cursor: "pointer" }}
      hoverStyle={{ backgroundColor: props.selected ? p().page : p().hover }}
    >
      <Text style={{ fontSize: 13, fontWeight: props.selected ? "600" : "400", color: props.selected ? p().text : p().textSoft }}>{props.label}</Text>
    </View>
  );
}

export function Sidebar(props: { tab: Tab; meta: DocumentMeta }) {
  const p = palette;
  const filter = signal("");
  let list!: ListRef;
  const outline = props.meta.outline;
  const minLevel = outline.reduce((m, o) => Math.min(m, o.level), 6);
  const visible = computed(() => {
    const f = filter().trim().toLowerCase();
    const indexed = outline.map((item, index) => ({ item, index }));
    return f ? indexed.filter((x) => x.item.text.toLowerCase().includes(f)) : indexed;
  });
  const marks = computed(() => (props.tab.builtin ? [] : bookmarksFor(props.tab.path)));
  const markedSet = computed(() => new Set(marks().map((b) => b.anchor)));
  const showBookmarks = () => settings().sidebarTab === "bookmarks";

  return (
    <View style={{ width: 280, backgroundColor: p().panel, borderEndWidth: 1, borderColor: p().divider }}>
      <View style={{ flexDirection: "row", gap: 4, margin: 10, padding: 3, borderRadius: 10, backgroundColor: p().chrome }}>
        <SidebarTabButton label={t("sidebar.contents")} selected={!showBookmarks()} onPress={() => updateSettings({ sidebarTab: "contents" })} />
        <SidebarTabButton label={t("sidebar.bookmarks")} selected={showBookmarks()} onPress={() => updateSettings({ sidebarTab: "bookmarks" })} />
      </View>
      <If
        when={!showBookmarks()}
        else={
          <If
            when={marks().length > 0}
            else={<Text style={{ margin: 18, fontSize: 13, color: p().textMuted, lineHeight: 20 }}>{t("sidebar.noBookmarks")}</Text>}
          >
            <Each of={marks()} key={(b: Bookmark) => b.anchor}>
              {(b) => (
                <View
                  role="link"
                  onPress={() => {
                    const target = outline.find((o) => o.anchor === b().anchor);
                    if (target) scrollToSection(target.section);
                  }}
                  style={{ flexDirection: "row", alignItems: "center", gap: 10, marginHorizontal: 8, paddingHorizontal: 10, minHeight: 34, borderRadius: 7, cursor: "pointer" }}
                  hoverStyle={{ backgroundColor: p().hover }}
                >
                  <Icon name="bookmark" size={14} color={p().accent} fill={p().accentSoft} />
                  <Text style={{ flex: 1, fontSize: 13, color: p().text, writingDirection: "auto", numberOfLines: 2 }}>{b().text}</Text>
                  <View
                    role="button"
                    accessibilityLabel={t("action.removeBookmark")}
                    onPress={() => toggleBookmark(props.tab.path, b())}
                    style={{ width: 24, height: 24, alignItems: "center", justifyContent: "center", borderRadius: 6, cursor: "pointer" }}
                    hoverStyle={{ backgroundColor: p().pressed }}
                  >
                    <Icon name="close" size={12} color={p().textMuted} />
                  </View>
                </View>
              )}
            </Each>
          </If>
        }
      >
        <If when={outline.length > 0} else={<Text style={{ margin: 18, fontSize: 13, color: p().textMuted }}>{t("sidebar.noHeadings")}</Text>}>
          {outline.length > 12 ? (
            <View style={{ marginHorizontal: 10, marginBottom: 6 }}>
              <Input
                value={filter()}
                placeholder={t("sidebar.filter")}
                onChange={(e) => filter.set(e.value)}
                style={{ height: 32, borderRadius: 8, paddingHorizontal: 10, fontSize: 13, backgroundColor: p().page, borderWidth: 1, borderColor: p().border, color: p().text }}
              />
            </View>
          ) : null}
          <List ref={list} each={visible()} estimatedItemSize={32} style={{ flex: 1 }} contentStyle={{ paddingBottom: 16 }}>
            {(entry) => (
              <OutlineRow
                item={() => entry().item}
                index={() => entry().index}
                path={props.tab.path}
                minLevel={minLevel}
                marked={(anchor) => !props.tab.builtin && markedSet().has(anchor)}
              />
            )}
          </List>
        </If>
      </If>
    </View>
  );
}
