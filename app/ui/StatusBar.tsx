// A quiet status bar: where you are, and what the document is.

import { Text, View } from "nsh";
import { formatBytes, formatNumber, t } from "./i18n";
import { currentHeading } from "./Reader";
import { activeTab, DEFAULT_SETTINGS, settings, zoomText } from "./state";
import { palette } from "./theme";

function Item(props: { text: string; onPress?: () => void; label?: string }) {
  const p = palette;
  return (
    <View
      role={props.onPress ? "button" : undefined}
      accessibilityLabel={props.label}
      onPress={props.onPress}
      style={{ paddingHorizontal: 8, height: 22, justifyContent: "center", borderRadius: 5, cursor: props.onPress ? "pointer" : undefined }}
      hoverStyle={props.onPress ? { backgroundColor: p().hover } : undefined}
    >
      <Text style={{ fontSize: 11.5, color: p().textMuted, numberOfLines: 1 }}>{props.text}</Text>
    </View>
  );
}

export function StatusBar() {
  const p = palette;
  const meta = () => activeTab()?.meta ?? null;
  const heading = () => {
    const m = meta();
    const i = currentHeading();
    return m && i >= 0 && m.outline[i] ? m.outline[i].text : "";
  };
  const zoom = () => `${Math.round((settings().fontSize / DEFAULT_SETTINGS.fontSize) * 100)}%`;
  return (
    <View style={{ height: 28, flexDirection: "row", alignItems: "center", paddingHorizontal: 8, gap: 2, backgroundColor: p().chrome, borderTopWidth: 1, borderColor: p().divider }}>
      <View style={{ flex: 1, flexDirection: "row", alignItems: "center" }}>
        {heading() ? <Item text={heading()} /> : null}
      </View>
      {meta() ? (
        <View style={{ flexDirection: "row", alignItems: "center" }}>
          <Item text={t("doc.words", { n: formatNumber(meta()!.stats.words) })} />
          <Item text={meta()!.direction === "rtl" ? t("doc.rtl") : t("doc.ltr")} />
          {activeTab()?.builtin ? null : <Item text={meta()!.encoding.toUpperCase()} />}
          {activeTab()?.builtin ? null : <Item text={formatBytes(meta()!.bytes)} />}
        </View>
      ) : null}
      <Item text={zoom()} label={t("action.zoomReset")} onPress={() => zoomText(0)} />
    </View>
  );
}
