// Interface translations. Strings live in `app/i18n/<lang>.json`, separate
// from the components; adding a language is adding a file and a line in
// `LANGUAGES` (see docs/localization-and-rtl.md).

import { computed, locale, signal } from "nsh";
import ar from "../i18n/ar.json";
import en from "../i18n/en.json";
import fr from "../i18n/fr.json";

export type Lang = "ar" | "en" | "fr";

export const LANGUAGES: { code: Lang; name: string; rtl: boolean }[] = [
  { code: "ar", name: "العربية", rtl: true },
  { code: "en", name: "English", rtl: false },
  { code: "fr", name: "Français", rtl: false },
];

const TABLES: Record<Lang, Record<string, string>> = { ar, en, fr };

/** The interface language. Arabic by default, per the product's identity. */
export const lang = signal<Lang>(initialLang());

function initialLang(): Lang {
  // First run: Arabic, unless the system's language is English or French.
  const system = (locale() || "ar").slice(0, 2).toLowerCase();
  return system === "en" || system === "fr" ? (system as Lang) : "ar";
}

/** Whether the interface lays out right to left. */
export const uiRtl = computed(() => LANGUAGES.find((l) => l.code === lang())?.rtl ?? false);

/**
 * The string for `key` in the current language, with `{name}` placeholders
 * filled from `vars`. Reactive: reading it in JSX follows language changes.
 */
export function t(key: string, vars?: Record<string, string | number>): string {
  const table = TABLES[lang()];
  let text = table[key] ?? TABLES.en[key] ?? key;
  if (vars) {
    for (const name in vars) text = text.split(`{${name}}`).join(String(vars[name]));
  }
  return text;
}

/** Numbers in the interface language's digits and grouping. */
export function formatNumber(n: number): string {
  try {
    // Western digits in Arabic too (as most Maghreb and technical readers
    // expect); grouping and separators follow the language.
    return new Intl.NumberFormat(lang() === "ar" ? "ar-u-nu-latn" : lang()).format(n);
  } catch {
    return String(n);
  }
}

/** A byte count as "12.4 MB". */
export function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${formatNumber(Math.round((bytes / 1024 / 1024) * 10) / 10)} ${t("unit.mb")}`;
  return `${formatNumber(Math.max(1, Math.round(bytes / 1024)))} ${t("unit.kb")}`;
}
