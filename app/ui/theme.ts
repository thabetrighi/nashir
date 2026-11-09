// Nashir's visual identity in code: palettes for the four reading themes,
// fonts, and the reading styles applied to NSH's TextView parts. Colors
// match brand/BRAND.md; Mermaid tokens match the Rust exporter
// (crates/nashir-engine/src/mermaid/svg.rs).

import { colorScheme, computed } from "nsh";
import { settings } from "./state";

export interface Palette {
  name: "light" | "sepia" | "dark";
  dark: boolean;
  /** Window chrome: title bar, tabs, status bar. */
  chrome: string;
  /** The sidebar. */
  panel: string;
  /** The reading surface. */
  page: string;
  border: string;
  divider: string;
  text: string;
  textSoft: string;
  textMuted: string;
  accent: string;
  accentStrong: string;
  accentSoft: string;
  onAccent: string;
  hover: string;
  pressed: string;
  selected: string;
  mark: string;
  markText: string;
  codeBg: string;
  codeBorder: string;
  danger: string;
  dangerSoft: string;
  warningSoft: string;
  warningText: string;
  overlay: string;
  shadow: string;
  /** Mermaid scene color tokens. */
  diagram: Record<string, string>;
}

const diagramLight: Record<string, string> = {
  bg: "#FFFFFF",
  node: "#EAF5F3",
  nodeBorder: "#0F766E",
  edge: "#475569",
  edgeLabel: "#FFFFFF",
  text: "#1F2937",
  muted: "#64748B",
  cluster: "#F6F5F0",
  clusterBorder: "#CBD5E1",
  note: "#FEF3C7",
  noteBorder: "#F59E0B",
  accent: "#0F766E",
  danger: "#DC2626",
  success: "#16A34A",
  onSeries: "#FFFFFF",
  s0: "#0F766E",
  s1: "#2563EB",
  s2: "#C2410C",
  s3: "#7C3AED",
  s4: "#BE185D",
  s5: "#0E7490",
  s6: "#4D7C0F",
  s7: "#A16207",
};

const diagramDark: Record<string, string> = {
  bg: "#16191D",
  node: "#1C2A29",
  nodeBorder: "#2DD4BF",
  edge: "#94A3B8",
  edgeLabel: "#16191D",
  text: "#E5E7EB",
  muted: "#94A3B8",
  cluster: "#1C2024",
  clusterBorder: "#3B4452",
  note: "#3A2F12",
  noteBorder: "#D97706",
  accent: "#14B8A6",
  danger: "#F87171",
  success: "#22C55E",
  onSeries: "#FFFFFF",
  s0: "#0D9488",
  s1: "#3B82F6",
  s2: "#EA580C",
  s3: "#8B5CF6",
  s4: "#DB2777",
  s5: "#0891B2",
  s6: "#65A30D",
  s7: "#CA8A04",
};

export const LIGHT: Palette = {
  name: "light",
  dark: false,
  chrome: "#F4F4F1",
  panel: "#F8F8F6",
  page: "#FFFFFF",
  border: "#E2E2DC",
  divider: "#ECECE7",
  text: "#1C2127",
  textSoft: "#3D4650",
  textMuted: "#6B7480",
  accent: "#0F766E",
  accentStrong: "#0B5E58",
  accentSoft: "#E3F2EF",
  onAccent: "#FFFFFF",
  hover: "#EAEAE5",
  pressed: "#DEDED8",
  selected: "#E3F2EF",
  mark: "#FDE68A",
  markText: "#1C2127",
  codeBg: "#F6F7F5",
  codeBorder: "#E4E6E1",
  danger: "#B42318",
  dangerSoft: "#FDECEA",
  warningSoft: "#FEF6E0",
  warningText: "#7A4B00",
  overlay: "rgba(20, 24, 28, 0.38)",
  shadow: "rgba(20, 24, 28, 0.12)",
  diagram: diagramLight,
};

export const SEPIA: Palette = {
  ...LIGHT,
  name: "sepia",
  chrome: "#EFE7D8",
  panel: "#F3ECDF",
  page: "#F8F1E3",
  border: "#DCCFB8",
  divider: "#E6DBC7",
  text: "#3B2F22",
  textSoft: "#57483A",
  textMuted: "#857360",
  hover: "#E7DDCB",
  pressed: "#DDD0B9",
  selected: "#E4E8D8",
  accentSoft: "#E1E7D9",
  codeBg: "#F1E8D7",
  codeBorder: "#E0D3BD",
  diagram: { ...diagramLight, bg: "#F8F1E3", edgeLabel: "#F8F1E3", cluster: "#F1E8D7", node: "#EAF0E4" },
};

export const DARK: Palette = {
  name: "dark",
  dark: true,
  chrome: "#111316",
  panel: "#15181B",
  page: "#191C20",
  border: "#2A2F35",
  divider: "#23272C",
  text: "#E6E8EB",
  textSoft: "#C3C8CE",
  textMuted: "#8B939C",
  accent: "#2DD4BF",
  accentStrong: "#5EEAD4",
  accentSoft: "#163532",
  onAccent: "#06201D",
  hover: "#22262B",
  pressed: "#2B3036",
  selected: "#163532",
  mark: "#7A5B00",
  markText: "#FFF5D6",
  codeBg: "#1F2328",
  codeBorder: "#2C3137",
  danger: "#F97066",
  dangerSoft: "#3A1C1A",
  warningSoft: "#33280F",
  warningText: "#F5D78E",
  overlay: "rgba(0, 0, 0, 0.55)",
  shadow: "rgba(0, 0, 0, 0.4)",
  diagram: { ...diagramDark, bg: "#191C20", edgeLabel: "#191C20" },
};

/** The palette in effect: the chosen theme, or the system's appearance. */
export const palette = computed<Palette>(() => {
  const choice = settings().theme;
  if (choice === "sepia") return SEPIA;
  if (choice === "dark") return DARK;
  if (choice === "light") return LIGHT;
  return colorScheme() === "dark" ? DARK : LIGHT;
});

export const READING_FONTS = [
  { id: "system", label: "Segoe UI", family: "Segoe UI" },
  { id: "tahoma", label: "Tahoma", family: "Tahoma" },
  { id: "arial", label: "Arial", family: "Arial" },
  { id: "sakkal", label: "Sakkal Majalla", family: "Sakkal Majalla" },
  { id: "traditional", label: "Traditional Arabic", family: "Traditional Arabic" },
  { id: "georgia", label: "Georgia", family: "Georgia" },
] as const;

export const CODE_FONTS = [
  { id: "cascadia", label: "Cascadia Mono", family: "Cascadia Mono" },
  { id: "consolas", label: "Consolas", family: "Consolas" },
  { id: "courier", label: "Courier New", family: "Courier New" },
] as const;

export const WIDTHS = { narrow: 640, medium: 760, wide: 920, full: 4000 } as const;

export function readingFont(): string {
  return READING_FONTS.find((f) => f.id === settings().font)?.family ?? "Segoe UI";
}

export function codeFont(): string {
  return CODE_FONTS.find((f) => f.id === settings().codeFont)?.family ?? "Cascadia Mono";
}

/** Styles for each part of NSH's TextView, from the reading settings. */
export const readingStyles = computed(() => {
  const p = palette();
  const s = settings();
  const size = s.fontSize;
  const lh = s.lineHeight;
  const heading = (scale: number, weight: "600" | "700") => ({
    fontSize: Math.round(size * scale),
    lineHeight: Math.round(size * scale * 1.3),
    fontWeight: weight,
    color: p.text,
    marginTop: Math.round(size * (scale > 1.4 ? 1.4 : 1.0)),
  });
  return {
    root: {
      fontSize: size,
      lineHeight: Math.round(size * lh),
      fontFamily: readingFont(),
      color: p.text,
      writingDirection: "auto" as const,
      textAlign: "auto" as const,
    },
    heading1: heading(1.9, "700"),
    heading2: heading(1.5, "700"),
    heading3: heading(1.25, "600"),
    heading4: heading(1.1, "600"),
    heading5: heading(1.0, "600"),
    heading6: { ...heading(0.95, "600"), color: p.textMuted },
    link: { color: p.accent },
    code: { fontFamily: codeFont(), fontSize: Math.round(size * 0.88), backgroundColor: p.codeBg },
    codeBlock: { backgroundColor: p.codeBg, borderColor: p.codeBorder, direction: "ltr" as const },
    codeBlockCode: {
      fontFamily: codeFont(),
      fontSize: Math.round(size * 0.84),
      lineHeight: Math.round(size * 0.84 * 1.55),
      writingDirection: "ltr" as const,
      textAlign: "left" as const,
    },
    blockquote: { color: p.textSoft, borderColor: p.border },
    highlight: { backgroundColor: p.mark, color: p.markText },
    tableHead: { backgroundColor: p.codeBg },
    tableCell: { borderColor: p.border },
    table: { borderColor: p.border },
    rule: { borderColor: p.border },
    listMarker: { color: p.textMuted },
    footnotes: { borderColor: p.divider },
    footnoteLabel: { color: p.textMuted },
  };
});
