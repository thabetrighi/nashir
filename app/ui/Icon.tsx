// Nashir's icon set: simple 24-point line icons drawn natively with Canvas
// and SVG path data (designed for Nashir; see brand/BRAND.md).

import { Canvas, Path2D } from "nsh";

const ICONS: Record<string, string[]> = {
  open: ["M3 7.5V18a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9.5a2 2 0 0 0-2-2h-7L10 5H5a2 2 0 0 0-2 2.5Z", "M3 11h18"],
  search: ["M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13Z", "M15.5 15.5 20 20"],
  settings: [
    "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
    "M19.4 13.5a7.6 7.6 0 0 0 0-3l2-1.6-2-3.4-2.4 1a7.5 7.5 0 0 0-2.6-1.5L14 2.5h-4l-.4 2.5a7.5 7.5 0 0 0-2.6 1.5l-2.4-1-2 3.4 2 1.6a7.6 7.6 0 0 0 0 3l-2 1.6 2 3.4 2.4-1a7.5 7.5 0 0 0 2.6 1.5l.4 2.5h4l.4-2.5a7.5 7.5 0 0 0 2.6-1.5l2.4 1 2-3.4-2-1.6Z",
  ],
  sidebar: ["M4 5h16a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z", "M9 5v14"],
  close: ["M6 6l12 12", "M18 6 6 18"],
  plus: ["M12 5v14", "M5 12h14"],
  minus: ["M5 12h14"],
  chevronDown: ["M6 9l6 6 6-6"],
  chevronUp: ["M6 15l6-6 6 6"],
  pin: ["M9 4h6l-1 6 3 3H7l3-3-1-6Z", "M12 13v7"],
  star: ["M12 3.8l2.5 5.1 5.6.8-4 4 1 5.6-5.1-2.7-5 2.7 1-5.6-4.1-4 5.6-.8L12 3.8Z"],
  bookmark: ["M7 4h10a1 1 0 0 1 1 1v15l-6-4-6 4V5a1 1 0 0 1 1-1Z"],
  list: ["M9 6h11", "M9 12h11", "M9 18h11", "M4.5 6h.01", "M4.5 12h.01", "M4.5 18h.01"],
  refresh: ["M20 11a8 8 0 1 0-2.3 5.7", "M20 4v7h-7"],
  copy: ["M9 9h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V10a1 1 0 0 1 1-1Z", "M16 9V5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3"],
  download: ["M12 4v11", "M7 10l5 5 5-5", "M5 20h14"],
  expand: ["M4 9V4h5", "M20 9V4h-5", "M4 15v5h5", "M20 15v5h-5"],
  zoomIn: ["M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13Z", "M15.5 15.5 20 20", "M10.5 8v5", "M8 10.5h5"],
  zoomOut: ["M10.5 17a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13Z", "M15.5 15.5 20 20", "M8 10.5h5"],
  sun: ["M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z", "M12 2v2", "M12 20v2", "M4.9 4.9l1.4 1.4", "M17.7 17.7l1.4 1.4", "M2 12h2", "M20 12h2", "M4.9 19.1l1.4-1.4", "M17.7 6.3l1.4-1.4"],
  moon: ["M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5Z"],
  book: ["M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5v-15Z", "M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5"],
  file: ["M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5Z", "M14 3v5h5", "M9 13h6", "M9 17h4"],
  folder: ["M3 7.5V18a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9.5a2 2 0 0 0-2-2h-7L10 5H5a2 2 0 0 0-2 2.5Z"],
  alert: ["M12 4 2.8 19.5h18.4L12 4Z", "M12 10v4", "M12 17h.01"],
  image: ["M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Z", "M4 16l5-5 4 4 3-3 4 4", "M15.5 9a1.5 1.5 0 1 0 0-.01"],
  shield: ["M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3Z", "M9 12l2 2 4-4"],
  keyboard: ["M3 6h18a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1Z", "M6 10h.01", "M10 10h.01", "M14 10h.01", "M18 10h.01", "M7 14h10"],
  more: ["M5 12h.01", "M12 12h.01", "M19 12h.01"],
  text: ["M5 7V5h14v2", "M12 5v14", "M9 19h6"],
};

export type IconName = keyof typeof ICONS;

const cache = new Map<string, InstanceType<typeof Path2D>>();
function path(d: string) {
  let p = cache.get(d);
  if (!p) {
    p = new Path2D(d);
    cache.set(d, p);
  }
  return p;
}

/** A line icon, `size` points square, stroked in `color`. */
export function Icon(props: { name: IconName; size?: number; color: string; fill?: string; strokeWidth?: number }) {
  const size = () => props.size ?? 18;
  return (
    <Canvas
      style={{ width: size(), height: size() }}
      accessibilityElementsHidden
      draw={(cx) => {
        const s = size();
        cx.scale(s / 24, s / 24);
        cx.lineCap = "round";
        cx.lineJoin = "round";
        cx.lineWidth = props.strokeWidth ?? 1.8;
        cx.strokeStyle = props.color;
        for (const d of ICONS[props.name] ?? []) {
          const p = path(d);
          if (props.fill) {
            cx.fillStyle = props.fill;
            cx.fill(p);
          }
          cx.stroke(p);
        }
      }}
    />
  );
}
