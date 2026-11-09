// In-document search (Ctrl+F): a floating bar with the result count,
// previous/next, and a list of snippets to jump to.

import { keymap, Text, View } from "nsh";
import { Input } from "nsh/base";
import { IconButton } from "./controls";
import { Icon } from "./Icon";
import { formatNumber, t } from "./i18n";
import type { DocumentMeta } from "./native";
import { scrollToSection } from "./Reader";
import { clearSearch, current, query, result, runSearch, searching, step } from "./search";
import { searchOpen } from "./state";
import { palette } from "./theme";

export function SearchBar(props: { meta: DocumentMeta }) {
  const p = palette;
  const go = (dir: number) => {
    const s = step(dir);
    if (s !== null) scrollToSection(s, 0.15);
  };
  const close = () => {
    searchOpen.set(false);
    clearSearch();
  };
  // Bound while the bar is open (removed with it).
  // `always`: they work while the search field has the focus too.
  keymap.bind({ escape: close, f3: () => go(1), "shift+f3": () => go(-1), "mod+g": () => go(1), "mod+shift+g": () => go(-1) }, { when: "always" });

  const count = () => {
    const r = result();
    if (searching()) return t("search.searching");
    if (!query().trim()) return "";
    if (!r || r.total === 0) return t("search.noResults");
    const total = r.truncated ? t("search.many", { total: formatNumber(r.total) }) : formatNumber(r.total);
    return t("search.results", { current: formatNumber(current() + 1), total });
  };

  return (
    <View
      style={{
        position: "absolute",
        top: 12,
        end: 20,
        width: 380,
        borderRadius: 12,
        backgroundColor: p().page,
        borderWidth: 1,
        borderColor: p().border,
        boxShadow: [{ x: 0, y: 6, blur: 18, color: p().shadow }],
        overflow: "hidden",
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", gap: 6, padding: 8 }}>
        <View style={{ paddingStart: 4 }}>
          <Icon name="search" size={16} color={p().textMuted} />
        </View>
        <Input
          testID="search-input"
          accessibilityLabel={t("search.placeholder")}
          autoFocus
          value={query()}
          placeholder={t("search.placeholder")}
          onChange={(e) => runSearch(props.meta.id, e.value, (s) => scrollToSection(s, 0.15))}
          onSubmit={() => go(1)}
          submitBehavior="submit"
          returnKeyType="search"
          style={{ flex: 1, height: 32, fontSize: 14, color: p().text, writingDirection: "auto", textAlign: "auto" }}
        />
        <Text style={{ fontSize: 12, color: p().textMuted, minWidth: 54, textAlign: "center" }}>{count()}</Text>
        <IconButton icon="chevronUp" label={t("search.prev")} size={16} onPress={() => go(-1)} disabled={!result()?.total} />
        <IconButton icon="chevronDown" label={t("search.next")} size={16} onPress={() => go(1)} disabled={!result()?.total} />
        <IconButton icon="close" label={t("action.close")} size={16} onPress={close} />
      </View>
      {result() && result()!.hits.length > 0 ? (
        <View style={{ maxHeight: 260, borderTopWidth: 1, borderColor: p().divider }}>
          {result()!
            .hits.slice(0, 8)
            .map((hit, i) => (
              <View
                role="button"
                onPress={() => {
                  current.set(i);
                  scrollToSection(hit.section, 0.15);
                }}
                style={{
                  paddingHorizontal: 14,
                  paddingVertical: 8,
                  backgroundColor: current() === i ? p().selected : "transparent",
                  cursor: "pointer",
                  gap: 2,
                }}
                hoverStyle={{ backgroundColor: p().hover }}
              >
                {hit.heading !== null && props.meta.outline[hit.heading] ? (
                  <Text style={{ fontSize: 11.5, color: p().textMuted, numberOfLines: 1, writingDirection: "auto" }}>
                    {props.meta.outline[hit.heading].text}
                  </Text>
                ) : null}
                <Text style={{ fontSize: 13, color: p().textSoft, numberOfLines: 2, writingDirection: "auto", textAlign: "auto" }}>
                  {hit.before}
                  <Text style={{ backgroundColor: p().mark, color: p().markText, fontWeight: "600" }}>{hit.matched}</Text>
                  {hit.after}
                </Text>
              </View>
            ))}
        </View>
      ) : null}
    </View>
  );
}
