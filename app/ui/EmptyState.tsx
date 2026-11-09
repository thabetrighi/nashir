// The welcome screen shown when no document is open.

import { Scroll, Text, View } from "nsh";
import { Button, Kbd } from "./controls";
import { Icon } from "./Icon";
import { t } from "./i18n";
import { Logo } from "./Logo";
import { openDialog } from "./actions";
import { Menu } from "nsh/desktop";
import { forgetRecent, openPaths, openWelcome, recent, type RecentEntry } from "./state";
import { palette } from "./theme";

const SHORTCUTS: [string, string][] = [
  ["Ctrl + O", "action.open"],
  ["Ctrl + F", "action.search"],
  ["Ctrl + B", "action.sidebar"],
  ["Ctrl + Tab", "action.nextTab"],
  ["Ctrl + +", "action.zoomIn"],
  ["F11", "action.fullscreen"],
];

function RecentRow(props: { entry: () => RecentEntry }) {
  const p = palette;
  const folder = () => props.entry().path.slice(0, Math.max(0, props.entry().path.length - props.entry().name.length - 1));
  return (
    <View
      role="button"
      accessibilityLabel={props.entry().name}
      // A missing file opens as a tab with a clear "not found" message.
      onPress={() => openPaths([props.entry().path])}
      onContextMenu={(e) =>
        Menu.popup([{ label: t("action.close"), onSelect: () => forgetRecent(props.entry().path) }], e.pagePosition)
      }
      style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingHorizontal: 12, paddingVertical: 9, borderRadius: 9, cursor: "pointer" }}
      hoverStyle={{ backgroundColor: p().hover }}
    >
      <Icon name="file" size={18} color={p().accent} />
      <View style={{ flex: 1, gap: 1 }}>
        <Text style={{ fontSize: 14, fontWeight: "600", color: p().text, numberOfLines: 1, writingDirection: "auto" }}>{props.entry().name}</Text>
        <Text style={{ fontSize: 12, color: p().textMuted, numberOfLines: 1, textOverflow: "ellipsis-start", direction: "ltr", textAlign: "auto" }}>{folder()}</Text>
      </View>
    </View>
  );
}

export function EmptyState(props: { dropping: boolean }) {
  const p = palette;
  return (
    <Scroll style={{ flex: 1, backgroundColor: p().page }} contentStyle={{ flexGrow: 1, alignItems: "center", justifyContent: "center", padding: 40 }}>
      <View style={{ width: "100%", maxWidth: 720, gap: 28 }}>
        <View style={{ alignItems: "center", gap: 14 }}>
          <Logo size={72} tile={p().accent} ink={p().dark ? "#06201D" : "#FFFFFF"} />
          <Text role="header" style={{ fontSize: 30, fontWeight: "700", color: p().text }}>
            {t("empty.title")}
          </Text>
          <Text style={{ fontSize: 15, color: p().textSoft, textAlign: "center" }}>{t("app.tagline")}</Text>
        </View>

        <View
          style={{
            borderRadius: 16,
            borderWidth: 2,
            borderStyle: "dashed",
            borderColor: props.dropping ? p().accent : p().border,
            backgroundColor: props.dropping ? p().accentSoft : p().panel,
            paddingVertical: 28,
            paddingHorizontal: 24,
            alignItems: "center",
            gap: 14,
          }}
        >
          <Icon name="open" size={30} color={props.dropping ? p().accent : p().textMuted} />
          <Text style={{ fontSize: 14.5, color: p().textSoft }}>{props.dropping ? t("empty.drop") : t("empty.subtitle")}</Text>
          <View style={{ flexDirection: "row", gap: 10, flexWrap: "wrap", justifyContent: "center" }}>
            <Button kind="primary" icon="open" label={t("action.open")} hint="Ctrl+O" onPress={openDialog} />
            <Button icon="book" label={t("action.welcome")} onPress={openWelcome} />
          </View>
        </View>

        <View style={{ flexDirection: "row", gap: 24, flexWrap: "wrap" }}>
          <View style={{ flex: 1, minWidth: 300, gap: 6 }}>
            <Text style={{ fontSize: 12.5, fontWeight: "700", color: p().textMuted, paddingHorizontal: 12 }}>{t("empty.recent")}</Text>
            {recent().length === 0 ? (
              <Text style={{ fontSize: 13.5, color: p().textMuted, paddingHorizontal: 12, paddingVertical: 8 }}>{t("empty.noRecent")}</Text>
            ) : (
              <Each of={recent().slice(0, 7)} key={(r: RecentEntry) => r.path}>
                {(entry) => <RecentRow entry={entry} />}
              </Each>
            )}
          </View>
          <View style={{ width: 250, gap: 10 }}>
            <Text style={{ fontSize: 12.5, fontWeight: "700", color: p().textMuted }}>{t("empty.shortcuts")}</Text>
            {SHORTCUTS.map(([keys, label]) => (
              <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", gap: 10 }}>
                <Text style={{ fontSize: 13, color: p().textSoft }}>{t(label)}</Text>
                <Kbd keys={keys} />
              </View>
            ))}
          </View>
        </View>

        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "center", gap: 8 }}>
          <Icon name="shield" size={15} color={p().textMuted} />
          <Text style={{ fontSize: 12.5, color: p().textMuted }}>{t("empty.privacy")}</Text>
        </View>
      </View>
    </Scroll>
  );
}
