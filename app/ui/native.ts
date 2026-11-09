// Typed access to the Rust native module (`app/native`, imported as
// `nashir/native`). Structured results cross as JSON strings; this module
// parses them into the types below so the rest of the UI never sees JSON.

import { Desktop, Diagrams, Library } from "nashir/native";

export type Direction = "rtl" | "ltr";

export interface OutlineItem {
  level: number;
  text: string;
  anchor: string;
  section: number;
}

export interface DocumentStats {
  words: number;
  characters: number;
  lines: number;
  headings: number;
  code_blocks: number;
  diagrams: number;
  reading_minutes: number;
}

export interface DocumentMeta {
  id: number;
  path: string;
  name: string;
  folder: string;
  bytes: number;
  encoding: string;
  lossy: boolean;
  modified: number;
  direction: Direction;
  /** One character per section: "r" or "l". */
  sectionDirections: string;
  outline: OutlineItem[];
  stats: DocumentStats;
  frontMatter: boolean;
}

export interface SearchHit {
  section: number;
  before: string;
  matched: string;
  after: string;
  heading: number | null;
}

export interface SearchResult {
  total: number;
  truncated: boolean;
  hits: SearchHit[];
}

export interface SceneShape {
  d: string;
  fill?: string;
  stroke?: string;
  strokeWidth: number;
  dash?: boolean;
}

export interface SceneLabel {
  x: number;
  y: number;
  w: number;
  h: number;
  text: string;
  size: number;
  weight: number;
  color: string;
  align: "center" | "start" | "end";
  dir: Direction;
  bg?: string;
}

export interface Scene {
  kind: string;
  width: number;
  height: number;
  shapes: SceneShape[];
  labels: SceneLabel[];
  title?: string;
}

export type MermaidResult =
  | { ok: true; scene: Scene }
  | { ok: false; error: "empty" | "unsupported" | "too_large" | "internal"; detail: string };

/** An error from `openDocument`, decoded from the Rust error code. */
export interface OpenError {
  code: "not_found" | "not_a_file" | "permission_denied" | "too_large" | "binary" | "io";
  bytes?: number;
  max?: number;
  message?: string;
}

function decodeError(e: unknown): OpenError {
  const text = String((e as { message?: string })?.message ?? e);
  const [code, a, b] = text.split(":");
  switch (code) {
    case "not_found":
    case "not_a_file":
    case "permission_denied":
    case "binary":
      return { code };
    case "too_large":
      return { code, bytes: Number(a), max: Number(b) };
    default:
      return { code: "io", message: text.replace(/^io:/, "") };
  }
}

export async function openDocument(path: string, maxMegabytes: number): Promise<DocumentMeta> {
  try {
    return JSON.parse(await Library.open(path, maxMegabytes)) as DocumentMeta;
  } catch (e) {
    throw decodeError(e);
  }
}

export async function openWelcome(lang: string): Promise<DocumentMeta> {
  return JSON.parse(await Library.openWelcome(lang)) as DocumentMeta;
}

export function sectionMarkdown(id: number, index: number, query: string): string {
  return query.trim() ? Library.highlighted(id, index, query) : Library.section(id, index);
}

export async function searchDocument(id: number, query: string): Promise<SearchResult> {
  return JSON.parse(await Library.search(id, query, 2000)) as SearchResult;
}

export const anchorSection = (id: number, anchor: string): number => Library.anchor(id, anchor);
export const headingAt = (id: number, section: number): number => Library.headingAt(id, section);
export const fileChanged = (id: number): boolean => Library.changed(id);
export const fileExists = (path: string): boolean => Library.exists(path);
export const closeDocument = (id: number): void => Library.close(id);
export const localImage = (id: number, src: string): Promise<string> => Library.image(id, src);

const sceneCache = new Map<string, MermaidResult>();

/** Lays out a Mermaid diagram (memoised by source: rows are recycled). */
export function layoutDiagram(source: string): MermaidResult {
  const cached = sceneCache.get(source);
  if (cached) return cached;
  let result: MermaidResult;
  try {
    result = JSON.parse(Diagrams.layout(source)) as MermaidResult;
  } catch {
    result = { ok: false, error: "internal", detail: "" };
  }
  if (sceneCache.size > 200) sceneCache.clear();
  sceneCache.set(source, result);
  return result;
}

export const exportDiagramSvg = (source: string, dark: boolean, name: string): Promise<boolean> =>
  Diagrams.exportSvg(source, dark, name);

export const pickMarkdown = (title: string, filterLabel: string): Promise<string[]> =>
  Desktop.pickMarkdown(title, filterLabel);
export const launchPaths = (): string[] => Desktop.launchPaths();
export const markdownPaths = (uris: string[]): string[] => Desktop.markdownPaths(uris);
export const revealInFolder = (path: string): Promise<void> => Desktop.reveal(path);
export const openWithSystem = (path: string): Promise<void> => Desktop.openWithSystem(path);
