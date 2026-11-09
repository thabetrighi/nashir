// Search within the active document. The query runs in Rust on a
// background thread (Arabic-aware, case-insensitive); results drive the
// reader's highlights and navigation.

import { computed, signal } from "nsh";
import { searchDocument, type SearchResult } from "./native";

export const query = signal("");
export const result = signal<SearchResult | null>(null);
export const current = signal(0);
export const searching = signal(false);
/** The document id the result belongs to. */
export const resultDoc = signal<number | null>(null);

/** Sections that contain matches (highlighted copies are rendered for them). */
export const matchedSections = computed(() => {
  const r = result();
  return new Set(r ? r.hits.map((h) => h.section) : []);
});

let seq = 0;
let timer: ReturnType<typeof setTimeout> | null = null;

/** Runs a search for `text` in document `id`, debounced. */
export function runSearch(id: number, text: string, onFirst?: (section: number) => void) {
  query.set(text);
  if (timer) clearTimeout(timer);
  if (!text.trim()) {
    seq++;
    result.set(null);
    searching.set(false);
    return;
  }
  searching.set(true);
  // Results of an older query are dropped, even if they arrive later.
  const mine = ++seq;
  timer = setTimeout(async () => {
    try {
      const r = await searchDocument(id, text);
      if (mine !== seq) return;
      result.set(r);
      resultDoc.set(id);
      current.set(0);
      if (r.hits.length && onFirst) onFirst(r.hits[0].section);
    } finally {
      if (mine === seq) searching.set(false);
    }
  }, 180);
}

export function clearSearch() {
  if (timer) clearTimeout(timer);
  seq++;
  query.set("");
  result.set(null);
  resultDoc.set(null);
  current.set(0);
  searching.set(false);
}

/** Moves to the next (`+1`) or previous (`-1`) result; returns its section. */
export function step(dir: number): number | null {
  const r = result();
  if (!r || !r.hits.length) return null;
  const next = (current() + dir + r.hits.length) % r.hits.length;
  current.set(next);
  return r.hits[next].section;
}

/** The active query, for documents the current result belongs to. */
export function highlightQuery(docId: number): string {
  return resultDoc() === docId && result() ? query() : "";
}
