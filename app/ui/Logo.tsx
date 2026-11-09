// Nashir's mark, drawn natively: a rounded tile in Nashir teal holding the
// letter ن (noon): its calligraphic bowl, an open page, and its rhombus
// nuqta, the point of focus. The same geometry as brand/logo-mark.svg.

import { Canvas, Path2D } from "nsh";

const TILE = "M14 2h36a12 12 0 0 1 12 12v36a12 12 0 0 1-12 12H14A12 12 0 0 1 2 50V14A12 12 0 0 1 14 2Z";
// The noon's bowl, calligraphic (thicker at its base, tapered at its tips).
const BOWL = "M13.5 26.5C13.5 40.5 21.5 50 32 50C42.5 50 50.5 40.5 50.5 26.5C50.5 25.4 49.6 24.6 48.5 24.6H47.7C46.6 24.6 45.8 25.5 45.8 26.6C45.6 37.4 39.9 44.4 32 44.4C24.1 44.4 18.4 37.4 18.2 26.6C18.2 25.5 17.4 24.6 16.3 24.6H15.5C14.4 24.6 13.5 25.4 13.5 26.5Z";
// The nuqta: the rhombus dot of Arabic calligraphy.
const DOT = "M32 11.6L36.6 16.2L32 20.8L27.4 16.2Z";

const tile = new Path2D(TILE);
const bowl = new Path2D(BOWL);
const dot = new Path2D(DOT);

export function Logo(props: { size: number; tile?: string; ink?: string }) {
  return (
    <Canvas
      style={{ width: props.size, height: props.size }}
      accessibilityElementsHidden
      draw={(cx) => {
        cx.scale(props.size / 64, props.size / 64);
        cx.fillStyle = props.tile ?? "#0F766E";
        cx.fill(tile);
        cx.fillStyle = props.ink ?? "#FFFFFF";
        cx.fill(bowl);
        cx.fill(dot);
      }}
    />
  );
}
