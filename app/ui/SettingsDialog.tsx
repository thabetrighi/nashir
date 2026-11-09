// Settings: appearance, reading, documents, privacy and about. Every change
// applies at once and is saved locally.

import { type JSX, Scroll, Text, View } from "nsh";
import { Dialog } from "nsh/base";
import { Button, IconButton, Segmented, Stepper, Toggle } from "./controls";
import { formatNumber, LANGUAGES, lang, type Lang, t } from "./i18n";
import { Logo } from "./Logo";
import { DEFAULT_SETTINGS, FONT_SIZES, settings, settingsOpen, updateSettings } from "./state";
import { CODE_FONTS, palette, READING_FONTS } from "./theme";

export const VERSION = "0.1.0";

function Section(props: { title: string; children: JSX.Element | JSX.Element[] }) {
  const p = palette;
  return (
    <View style={{ gap: 14, paddingVertical: 18, borderBottomWidth: 1, borderColor: p().divider }}>
      <Text role="header" style={{ fontSize: 12.5, fontWeight: "700", color: p().accent, letterSpacing: 0.3 }}>
        {props.title}
      </Text>
      {props.children}
    </View>
  );
}

function Row(props: { label: string; hint?: string; children: JSX.Element | JSX.Element[] }) {
  const p = palette;
  return (
    <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", gap: 16, flexWrap: "wrap" }}>
      <View style={{ flex: 1, minWidth: 180, gap: 2 }}>
        <Text style={{ fontSize: 14, color: p().text }}>{props.label}</Text>
        {props.hint ? <Text style={{ fontSize: 12, color: p().textMuted, lineHeight: 17 }}>{props.hint}</Text> : null}
      </View>
      {props.children}
    </View>
  );
}

function Chips(props: { value: string; options: readonly { id: string; label: string; family: string }[]; onChange: (id: string) => void; label: string }) {
  const p = palette;
  return (
    <View accessibilityLabel={props.label} style={{ flexDirection: "row", flexWrap: "wrap", gap: 6, maxWidth: 360 }}>
      {props.options.map((o) => (
        <View
          role="button"
          accessibilityLabel={o.label}
          accessibilityState={{ selected: props.value === o.id }}
          onPress={() => props.onChange(o.id)}
          style={{
            paddingHorizontal: 12,
            height: 30,
            borderRadius: 15,
            justifyContent: "center",
            borderWidth: 1,
            borderColor: props.value === o.id ? p().accent : p().border,
            backgroundColor: props.value === o.id ? p().accentSoft : "transparent",
            cursor: "pointer",
          }}
          hoverStyle={{ backgroundColor: props.value === o.id ? p().accentSoft : p().hover }}
        >
          <Text style={{ fontSize: 13, fontFamily: o.family, color: props.value === o.id ? p().accent : p().text }}>{o.label}</Text>
        </View>
      ))}
    </View>
  );
}

export function SettingsDialog() {
  const p = palette;
  const s = settings;
  return (
    <Dialog
      open={settingsOpen()}
      animation="fade"
      onOpenChange={(e) => settingsOpen.set(e.open)}
      backdropStyle={{ backgroundColor: p().overlay }}
      style={{
        width: 620,
        maxWidth: "92%",
        maxHeight: "88%",
        alignSelf: "center",
        marginVertical: "auto",
        borderRadius: 16,
        backgroundColor: p().page,
        borderWidth: 1,
        borderColor: p().border,
        overflow: "hidden",
        boxShadow: [{ x: 0, y: 18, blur: 48, color: p().shadow }],
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", paddingHorizontal: 22, paddingVertical: 14, borderBottomWidth: 1, borderColor: p().divider }}>
        <Text role="header" style={{ fontSize: 18, fontWeight: "700", color: p().text }}>
          {t("settings.title")}
        </Text>
        <IconButton icon="close" label={t("action.close")} onPress={() => settingsOpen.set(false)} />
      </View>
      <Scroll style={{ flexShrink: 1 }} contentStyle={{ paddingHorizontal: 22, paddingBottom: 22 }}>
        <Section title={t("settings.appearance")}>
          <Row label={t("settings.theme")}>
            <Segmented
              label={t("settings.theme")}
              value={s().theme}
              onChange={(theme) => updateSettings({ theme })}
              options={[
                { value: "system", label: t("settings.theme.system") },
                { value: "light", label: t("settings.theme.light") },
                { value: "sepia", label: t("settings.theme.sepia") },
                { value: "dark", label: t("settings.theme.dark") },
              ]}
            />
          </Row>
          <Row label={t("settings.language")}>
            <Segmented<Lang>
              label={t("settings.language")}
              value={lang()}
              onChange={(l) => lang.set(l)}
              options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
            />
          </Row>
        </Section>

        <Section title={t("settings.reading")}>
          <Row label={t("settings.fontSize")}>
            <Stepper
              label={t("settings.fontSize")}
              value={formatNumber(s().fontSize)}
              onDecrease={() => updateSettings({ fontSize: Math.max(FONT_SIZES.min, s().fontSize - 1) })}
              onIncrease={() => updateSettings({ fontSize: Math.min(FONT_SIZES.max, s().fontSize + 1) })}
            />
          </Row>
          <Row label={t("settings.lineHeight")}>
            <Stepper
              label={t("settings.lineHeight")}
              value={formatNumber(Math.round(s().lineHeight * 10) / 10)}
              onDecrease={() => updateSettings({ lineHeight: Math.max(1.2, Math.round((s().lineHeight - 0.1) * 10) / 10) })}
              onIncrease={() => updateSettings({ lineHeight: Math.min(2.6, Math.round((s().lineHeight + 0.1) * 10) / 10) })}
            />
          </Row>
          <Row label={t("settings.width")}>
            <Segmented
              label={t("settings.width")}
              value={s().width}
              onChange={(width) => updateSettings({ width })}
              options={[
                { value: "narrow", label: t("settings.width.narrow") },
                { value: "medium", label: t("settings.width.medium") },
                { value: "wide", label: t("settings.width.wide") },
                { value: "full", label: t("settings.width.full") },
              ]}
            />
          </Row>
          <Row label={t("settings.font")}>
            <Chips label={t("settings.font")} value={s().font} options={READING_FONTS} onChange={(font) => updateSettings({ font })} />
          </Row>
          <Row label={t("settings.codeFont")}>
            <Chips label={t("settings.codeFont")} value={s().codeFont} options={CODE_FONTS} onChange={(codeFont) => updateSettings({ codeFont })} />
          </Row>
          <Row label={t("action.comfort")}>
            <Toggle label={t("action.comfort")} value={s().comfort} onChange={(comfort) => updateSettings({ comfort })} />
          </Row>
        </Section>

        <Section title={t("settings.document")}>
          <Row label={t("settings.direction")}>
            <Segmented
              label={t("settings.direction")}
              value={s().direction}
              onChange={(direction) => updateSettings({ direction })}
              options={[
                { value: "auto", label: t("settings.direction.auto") },
                { value: "rtl", label: t("settings.direction.rtl") },
                { value: "ltr", label: t("settings.direction.ltr") },
              ]}
            />
          </Row>
          <Row label={t("settings.remoteImages")} hint={t("settings.remoteImages.hint")}>
            <Toggle label={t("settings.remoteImages")} value={s().remoteImages} onChange={(remoteImages) => updateSettings({ remoteImages })} />
          </Row>
          <Row label={t("settings.restore")}>
            <Toggle label={t("settings.restore")} value={s().restoreSession} onChange={(restoreSession) => updateSettings({ restoreSession })} />
          </Row>
          <Row label={t("settings.maxSize")}>
            <Segmented
              label={t("settings.maxSize")}
              value={String(s().maxFileMB)}
              onChange={(v) => updateSettings({ maxFileMB: Number(v) })}
              options={["16", "64", "128", "256"].map((v) => ({ value: v, label: `${v} MB` }))}
            />
          </Row>
        </Section>

        <Section title={t("settings.privacy")}>
          <Text style={{ fontSize: 13.5, color: p().textSoft, lineHeight: 21 }}>{t("settings.privacy.text")}</Text>
        </Section>

        <View style={{ flexDirection: "row", alignItems: "center", gap: 14, paddingTop: 18 }}>
          <Logo size={40} tile={p().accent} ink={p().dark ? "#06201D" : "#FFFFFF"} />
          <View style={{ flex: 1, gap: 2 }}>
            <Text style={{ fontSize: 15, fontWeight: "700", color: p().text }}>{`${t("app.name")} · Nashir`}</Text>
            <Text style={{ fontSize: 12.5, color: p().textMuted }}>{t("settings.version", { version: VERSION })}</Text>
          </View>
          <Button compact kind="ghost" label={t("settings.reset")} onPress={() => updateSettings({ ...DEFAULT_SETTINGS })} />
        </View>
      </Scroll>
    </Dialog>
  );
}
