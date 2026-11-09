// The reading surface: a virtualized List of document sections, each a
// native TextView. Only sections near the viewport exist, so a 50 MB
// document scrolls as smoothly as a short one. Each row asks Rust for its
// own Markdown when it appears.

import { computed, Image, type ListRef, List, openUrl, pixelRatio, signal, Text, View } from "nsh";
import { type MarkdownExtension, TextView, type TextViewCustomNode, type TextViewImage } from "nsh/base";
import { Button, IconButton } from "./controls";
import { Icon } from "./Icon";
import { formatBytes, formatNumber, t } from "./i18n";
import { MermaidView } from "./MermaidView";
import { anchorSection, type DocumentMeta, headingAt, localImage, sectionMarkdown } from "./native";
import { highlightQuery, matchedSections } from "./search";
import {
  dismissChange,
  openPaths,
  reloadTab,
  settings,
  setTabSection,
  tabSection,
  type Tab,
  closeTab,
} from "./state";
import { palette, readingStyles, WIDTHS } from "./theme";

const EXTENSIONS: MarkdownExtension[] = [{ name: "mermaid", language: "mermaid" }];

/** The reader of the active tab registers how to scroll it. */
let scroller: ((section: number, position?: number) => void) | null = null;
export function scrollToSection(section: number, position = 0) {
  scroller?.(section, position);
}

/** The outline entry under the top of the reader (for the sidebar). */
export const currentHeading = signal(-1);

function hostOf(url: string): string {
  const m = /^[a-z]+:\/\/([^/]+)/i.exec(url);
  return m ? m[1] : url;
}

function DocImage(props: { docId: number; image: TextViewImage }) {
  const p = palette;
  const src = props.image.src;
  const remote = /^https?:\/\//i.test(src);
  const data = /^data:/i.test(src);
  const uri = signal<string | null>(data || (remote && settings().remoteImages) ? src : null);
  const failed = signal(false);
  const natural = signal<{ w: number; h: number } | null>(null);
  if (!remote && !data) {
    localImage(props.docId, src)
      .then((u) => (u ? uri.set(u) : failed.set(true)))
      .catch(() => failed.set(true));
  }
  const box = { borderRadius: 10, borderWidth: 1, borderColor: p().border, padding: 14, marginVertical: 8, flexDirection: "row" as const, alignItems: "center" as const, gap: 10 };
  return (
    <View style={{ marginVertical: 6 }}>
      {failed() ? (
        <View style={box}>
          <Icon name="image" size={18} color={p().textMuted} />
          <Text style={{ color: p().textMuted, fontSize: 14, flex: 1 }}>{props.image.alt || t("image.missing")}</Text>
        </View>
      ) : uri() ? (
        <Image
          src={uri()!}
          accessibilityLabel={props.image.alt ?? ""}
          onLoad={(e) => natural.set({ w: e.size.width / pixelRatio(), h: e.size.height / pixelRatio() })}
          onError={() => failed.set(true)}
          style={{
            width: "100%",
            maxWidth: natural()?.w,
            aspectRatio: natural() ? natural()!.w / Math.max(1, natural()!.h) : 16 / 9,
            borderRadius: 8,
            objectFit: "contain",
            alignSelf: "center",
          }}
        />
      ) : remote ? (
        <View style={{ ...box, backgroundColor: p().codeBg }}>
          <Icon name="image" size={18} color={p().textMuted} />
          <Text style={{ color: p().textSoft, fontSize: 14, flex: 1, numberOfLines: 1 }}>
            {props.image.alt ? `${props.image.alt} — ` : ""}
            {t("image.remote", { host: hostOf(src) })}
          </Text>
          <Button compact label={t("image.load")} onPress={() => uri.set(src)} />
        </View>
      ) : (
        <View style={{ ...box, height: 120 }} />
      )}
    </View>
  );
}

function FrontMatter(props: { yaml: string }) {
  const p = palette;
  const rows = props.yaml
    .split("\n")
    .map((l) => l.match(/^([\w؀-ۿ -]+):\s*(.*)$/))
    .filter((m): m is RegExpMatchArray => !!m && !!m[2]);
  return (
    <View style={{ borderRadius: 10, borderWidth: 1, borderColor: p().border, backgroundColor: p().codeBg, padding: 14, gap: 6, marginBottom: 12 }}>
      <Text style={{ fontSize: 12, fontWeight: "700", color: p().textMuted, letterSpacing: 0.4 }}>{t("doc.frontMatter")}</Text>
      {rows.map((m) => (
        <View style={{ flexDirection: "row", gap: 10 }}>
          <Text style={{ fontSize: 13, color: p().textMuted, minWidth: 90 }}>{m[1].trim()}</Text>
          <Text style={{ fontSize: 13, color: p().text, flex: 1, writingDirection: "auto" }}>{m[2].replace(/^["']|["']$/g, "")}</Text>
        </View>
      ))}
    </View>
  );
}

function SectionRow(props: { meta: DocumentMeta; index: () => number; onLink: (url: string) => void }) {
  const p = palette;
  const markdown = computed(() => {
    const i = props.index();
    const q = matchedSections().has(i) ? highlightQuery(props.meta.id) : "";
    return sectionMarkdown(props.meta.id, i, q);
  });
  const direction = () => {
    const choice = settings().direction;
    if (choice !== "auto") return choice;
    return props.meta.sectionDirections[props.index()] === "r" ? "rtl" : "ltr";
  };
  return (
    <View style={{ alignItems: "center", paddingHorizontal: 40 }}>
      <View style={{ width: "100%", maxWidth: WIDTHS[settings().width], direction: direction(), paddingVertical: 4 }}>
        <TextView
          markdown={markdown()}
          selectable
          options={{ frontmatter: props.index() === 0 && props.meta.frontMatter }}
          extensions={EXTENSIONS}
          styles={readingStyles()}
          classes={{ root: "gap-4" }}
          codeBlockActions={[{ id: "copy", label: t("action.copy") }]}
          onLinkPress={({ url }) => props.onLink(url)}
          renderImage={(image) => <DocImage docId={props.meta.id} image={image} />}
          renderCustom={(node: TextViewCustomNode) =>
            node.name === "mermaid" ? (
              <MermaidView source={node.text ?? ""} />
            ) : node.name === "frontmatter" ? (
              <FrontMatter yaml={node.text ?? ""} />
            ) : (
              <Text style={{ color: p().textMuted }}>{node.text ?? ""}</Text>
            )
          }
        />
      </View>
    </View>
  );
}

function DocHeader(props: { tab: Tab; meta: DocumentMeta }) {
  const p = palette;
  const s = props.meta.stats;
  const parts = [
    t("doc.minutes", { n: formatNumber(s.reading_minutes) }),
    t("doc.words", { n: formatNumber(s.words) }),
    props.tab.builtin ? null : formatBytes(props.meta.bytes),
  ].filter(Boolean);
  return (
    <View style={{ alignItems: "center", paddingHorizontal: 40, paddingTop: 28 }}>
      <View style={{ width: "100%", maxWidth: WIDTHS[settings().width], gap: 6, paddingBottom: 18, borderBottomWidth: 1, borderColor: p().divider, marginBottom: 8 }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
          <Icon name="file" size={15} color={p().textMuted} />
          <Text style={{ fontSize: 12.5, color: p().textMuted, flex: 1, direction: "ltr", textAlign: "auto", numberOfLines: 1 }}>
            {props.tab.builtin ? t("app.name") : props.meta.folder}
          </Text>
        </View>
        <Text style={{ fontSize: 12.5, color: p().textMuted }}>{parts.join("  ·  ")}</Text>
      </View>
    </View>
  );
}

function Banner(props: { text: string; tone: "warning" | "error"; actions: { label: string; onPress: () => void; primary?: boolean }[] }) {
  const p = palette;
  return (
    <View
      role="alert"
      style={{
        flexDirection: "row",
        alignItems: "center",
        gap: 12,
        paddingHorizontal: 16,
        paddingVertical: 10,
        backgroundColor: props.tone === "error" ? p().dangerSoft : p().warningSoft,
        borderBottomWidth: 1,
        borderColor: p().border,
      }}
    >
      <Icon name="alert" size={18} color={props.tone === "error" ? p().danger : p().warningText} />
      <Text style={{ flex: 1, fontSize: 13.5, color: props.tone === "error" ? p().danger : p().warningText }}>{props.text}</Text>
      {props.actions.map((a) => (
        <Button compact kind={a.primary ? "primary" : "ghost"} label={a.label} onPress={a.onPress} />
      ))}
    </View>
  );
}

export function LoadingView() {
  const p = palette;
  return (
    <View style={{ flex: 1, alignItems: "center", paddingTop: 80, gap: 16, backgroundColor: p().page }}>
      {[0.9, 0.75, 0.82, 0.6].map((w) => (
        <View style={{ width: "60%", maxWidth: 640, height: 14, borderRadius: 7, backgroundColor: p().hover, opacity: w }} />
      ))}
      <Text style={{ color: p().textMuted, fontSize: 14, marginTop: 12 }}>{t("doc.loading")}</Text>
    </View>
  );
}

export function ErrorView(props: { tab: Tab }) {
  const p = palette;
  const e = props.tab.error!;
  const message =
    e.code === "too_large"
      ? t("error.too_large", { size: formatBytes(e.bytes ?? 0), max: formatBytes(e.max ?? 0) })
      : e.code === "io"
        ? t("error.io", { message: e.message ?? "" })
        : t(`error.${e.code}`);
  return (
    <View style={{ flex: 1, alignItems: "center", justifyContent: "center", padding: 32, backgroundColor: p().page }}>
      <View style={{ maxWidth: 480, alignItems: "center", gap: 14 }}>
        <View style={{ width: 56, height: 56, borderRadius: 28, backgroundColor: p().dangerSoft, alignItems: "center", justifyContent: "center" }}>
          <Icon name="alert" size={26} color={p().danger} />
        </View>
        <Text role="header" style={{ fontSize: 20, fontWeight: "700", color: p().text, textAlign: "center" }}>
          {t("error.title")}
        </Text>
        <Text style={{ fontSize: 14.5, color: p().textSoft, textAlign: "center", lineHeight: 22 }}>{message}</Text>
        <Text style={{ fontSize: 12.5, color: p().textMuted, textAlign: "center", direction: "ltr", numberOfLines: 2 }}>
          {props.tab.path}
        </Text>
        <View style={{ flexDirection: "row", gap: 10, marginTop: 6 }}>
          <Button kind="primary" icon="refresh" label={t("action.retry")} onPress={() => reloadTab(props.tab.key)} />
          <Button label={t("action.closeTab")} onPress={() => closeTab(props.tab.key, true)} />
        </View>
      </View>
    </View>
  );
}

export function Reader(props: { tab: Tab }) {
  const p = palette;
  const meta = props.tab.meta!;
  const count = meta.sectionDirections.length || 1;
  const sections = Array.from({ length: count }, (_, i) => i);
  let list!: ListRef;

  scroller = (section, position = 0) => {
    list?.scrollToIndex({ index: Math.max(0, Math.min(count - 1, section)), animated: true, viewPosition: position, viewOffset: 12 });
  };

  const onLink = (url: string) => {
    if (url.startsWith("#footnote-")) return;
    if (url.startsWith("#")) {
      const s = anchorSection(meta.id, decodeURIComponent(url.slice(1)));
      if (s >= 0) scrollToSection(s);
      return;
    }
    if (/^(https?|mailto):/i.test(url)) {
      openUrl(url).catch(() => undefined);
      return;
    }
    // A relative link to another Markdown file: open it in a tab.
    const [file] = url.split("#");
    if (/\.(md|markdown|mdown|mkd|mkdn)$/i.test(file) && !props.tab.builtin) {
      const sep = meta.folder.includes("\\") ? "\\" : "/";
      const target = /^([a-z]:|\/|\\\\)/i.test(file) ? file : `${meta.folder}${sep}${decodeURIComponent(file).replace(/[\\/]/g, sep)}`;
      openPaths([target]);
    }
  };

  return (
    <View style={{ flex: 1, backgroundColor: p().page }}>
      {props.tab.changed ? (
        <Banner
          tone="warning"
          text={t("doc.changed")}
          actions={[
            { label: t("action.dismiss"), onPress: () => dismissChange(props.tab.key) },
            { label: t("action.reload"), onPress: () => reloadTab(props.tab.key), primary: true },
          ]}
        />
      ) : null}
      {meta.lossy ? <Banner tone="warning" text={t("doc.lossy")} actions={[]} /> : null}
      <List
        ref={list}
        each={sections}
        estimatedItemSize={320}
        initialScrollIndex={Math.min(tabSection(props.tab.key), count - 1)}
        header={<DocHeader tab={props.tab} meta={meta} />}
        footer={<View style={{ height: 160 }} />}
        viewabilityConfig={{ itemVisibleThreshold: 0.01 }}
        onViewableItemsChange={({ viewableItems }) => {
          const first = viewableItems.reduce((min, v) => Math.min(min, v.index ?? min), Number.MAX_SAFE_INTEGER);
          if (first !== Number.MAX_SAFE_INTEGER) {
            setTabSection(props.tab.key, first);
            currentHeading.set(headingAt(meta.id, first));
          }
        }}
        style={{ flex: 1 }}
        testID="reader"
      >
        {(_item, index) => <SectionRow meta={meta} index={index} onLink={onLink} />}
      </List>
      <IconButtonFloating />
    </View>
  );
}

/** A small "back to top" affordance shown at the reader's corner. */
function IconButtonFloating() {
  const p = palette;
  return (
    <View style={{ position: "absolute", bottom: 18, end: 22, borderRadius: 10, backgroundColor: p().chrome, borderWidth: 1, borderColor: p().border }}>
      <IconButton icon="chevronUp" label="↑" onPress={() => scrollToSection(0)} />
    </View>
  );
}
