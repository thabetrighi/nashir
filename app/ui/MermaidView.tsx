// A Mermaid diagram drawn natively: the Rust engine lays it out into a
// scene; shapes are drawn on a Canvas from SVG path data and labels are
// native Text views placed over it, so Arabic is shaped by the system's
// text engine, selectable, and read by screen readers.

import { Canvas, Clipboard, Path2D, Scroll, signal, Text, View } from "nsh";
import { Dialog } from "nsh/base";
import { IconButton } from "./controls";
import { t } from "./i18n";
import { exportDiagramSvg, layoutDiagram, type Scene } from "./native";
import { toast } from "./state";
import { palette } from "./theme";

const ZOOMS = [0.5, 0.67, 0.8, 1, 1.25, 1.5, 2, 2.5, 3];

function SceneView(props: { scene: Scene; zoom: number }) {
  const p = palette;
  const color = (token: string | undefined) => (token ? (p().diagram[token] ?? p().diagram.text) : undefined);
  const paths = props.scene.shapes.map((s) => ({ shape: s, path: new Path2D(s.d) }));
  return (
    <View style={{ width: props.scene.width * props.zoom, height: props.scene.height * props.zoom }}>
      <Canvas
        style={{ position: "absolute", left: 0, top: 0, width: props.scene.width * props.zoom, height: props.scene.height * props.zoom }}
        accessibilityElementsHidden
        draw={(cx) => {
          const z = props.zoom;
          cx.scale(z, z);
          cx.lineJoin = "round";
          cx.lineCap = "round";
          for (const { shape, path } of paths) {
            const fill = color(shape.fill);
            if (fill) {
              cx.fillStyle = fill;
              cx.fill(path);
            }
            const stroke = color(shape.stroke);
            if (stroke && shape.strokeWidth > 0) {
              cx.strokeStyle = stroke;
              cx.lineWidth = shape.strokeWidth;
              cx.setLineDash(shape.dash ? [5, 4] : []);
              cx.stroke(path);
            }
          }
        }}
      />
      {props.scene.labels.map((l) => (
        <View
          style={{
            position: "absolute",
            left: l.x * props.zoom,
            top: l.y * props.zoom,
            width: l.w * props.zoom,
            height: l.h * props.zoom,
            justifyContent: "center",
            borderRadius: 3,
            backgroundColor: l.bg ? color(l.bg) : undefined,
            direction: l.dir,
          }}
        >
          <Text
            style={{
              fontSize: l.size * props.zoom,
              lineHeight: l.size * 1.4 * props.zoom,
              fontWeight: String(l.weight) as "400",
              color: color(l.color),
              textAlign: l.align === "center" ? "center" : l.align,
              writingDirection: l.dir,
              userSelect: "text",
            }}
          >
            {l.text}
          </Text>
        </View>
      ))}
    </View>
  );
}

function Toolbar(props: {
  zoom: () => number;
  setZoom: (z: number) => void;
  source: string;
  kind: string;
  onExpand?: () => void;
}) {
  const p = palette;
  const step = (dir: number) => {
    const i = ZOOMS.findIndex((z) => z >= props.zoom() - 0.001);
    const next = ZOOMS[Math.min(ZOOMS.length - 1, Math.max(0, (i < 0 ? 3 : i) + dir))];
    props.setZoom(next);
  };
  return (
    <View style={{ flexDirection: "row", alignItems: "center", gap: 2 }}>
      <IconButton icon="zoomOut" label={t("diagram.zoomOut")} size={16} onPress={() => step(-1)} />
      <View
        role="button"
        accessibilityLabel={t("diagram.fit")}
        onPress={() => props.setZoom(1)}
        style={{ minWidth: 46, height: 26, alignItems: "center", justifyContent: "center", borderRadius: 6, cursor: "pointer" }}
        hoverStyle={{ backgroundColor: p().hover }}
      >
        <Text style={{ fontSize: 12, color: p().textSoft, fontWeight: "600", direction: "ltr" }}>{`${Math.round(props.zoom() * 100)}%`}</Text>
      </View>
      <IconButton icon="zoomIn" label={t("diagram.zoomIn")} size={16} onPress={() => step(1)} />
      <View style={{ width: 1, height: 18, backgroundColor: p().divider, marginHorizontal: 4 }} />
      <IconButton
        icon="copy"
        label={t("diagram.copySource")}
        size={16}
        onPress={async () => {
          await Clipboard.writeText(props.source);
          toast(t("action.copied"));
        }}
      />
      <IconButton
        icon="download"
        label={t("diagram.exportSvg")}
        size={16}
        onPress={async () => {
          try {
            if (await exportDiagramSvg(props.source, palette().dark, `${props.kind}-diagram`)) toast(t("diagram.exported"));
          } catch (e) {
            toast(String(e), "error");
          }
        }}
      />
      {props.onExpand ? <IconButton icon="expand" label={t("diagram.expand")} size={16} onPress={props.onExpand} /> : null}
    </View>
  );
}

export function MermaidView(props: { source: string }) {
  const p = palette;
  const result = layoutDiagram(props.source);
  const zoom = signal(1);
  const expanded = signal(false);
  const expandedZoom = signal(1);

  if (!result.ok) {
    const message =
      result.error === "unsupported"
        ? t("diagram.unsupported", { kind: result.detail })
        : result.error === "empty"
          ? t("diagram.empty")
          : result.error === "too_large"
            ? t("diagram.tooLarge")
            : t("diagram.internal");
    return (
      <View style={{ borderRadius: 10, borderWidth: 1, borderColor: p().border, backgroundColor: p().warningSoft, padding: 14, gap: 10, marginVertical: 8 }}>
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", gap: 8 }}>
          <Text style={{ color: p().warningText, fontWeight: "600", fontSize: 14, flex: 1 }}>{message}</Text>
          <IconButton
            icon="copy"
            label={t("diagram.copySource")}
            size={16}
            onPress={async () => {
              await Clipboard.writeText(props.source);
              toast(t("action.copied"));
            }}
          />
        </View>
        <View style={{ direction: "ltr", backgroundColor: p().codeBg, borderRadius: 8, padding: 10 }}>
          <Text style={{ fontFamily: "Cascadia Mono", fontSize: 13, color: p().textSoft, writingDirection: "ltr", userSelect: "text" }}>{props.source}</Text>
        </View>
      </View>
    );
  }

  const scene = result.scene;
  return (
    <View
      accessibilityLabel={t("diagram.label", { kind: scene.kind })}
      style={{ borderRadius: 12, borderWidth: 1, borderColor: p().border, backgroundColor: p().diagram.bg, marginVertical: 10, overflow: "hidden" }}
    >
      <View style={{ direction: "ltr", flexDirection: "row", justifyContent: "flex-end", paddingHorizontal: 6, paddingTop: 6 }}>
        <Toolbar zoom={zoom} setZoom={(z) => zoom.set(z)} source={props.source} kind={scene.kind} onExpand={() => expanded.set(true)} />
      </View>
      <Scroll horizontal style={{ direction: "ltr" }} contentStyle={{ flexGrow: 1, justifyContent: "center", paddingHorizontal: 12, paddingBottom: 12 }}>
        <SceneView scene={scene} zoom={zoom()} />
      </Scroll>
      <Dialog
        open={expanded()}
        animation="fade"
        onOpenChange={(e) => expanded.set(e.open)}
        backdropStyle={{ backgroundColor: p().overlay }}
        style={{ margin: 32, flex: 1, borderRadius: 14, backgroundColor: p().diagram.bg, overflow: "hidden", borderWidth: 1, borderColor: p().border }}
      >
        <View style={{ direction: "ltr", flexDirection: "row", alignItems: "center", justifyContent: "space-between", padding: 8, borderBottomWidth: 1, borderColor: p().divider }}>
          <Text style={{ fontSize: 14, fontWeight: "600", color: p().text, paddingHorizontal: 8 }}>{scene.title ?? t("diagram.label", { kind: scene.kind })}</Text>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 4 }}>
            <Toolbar zoom={expandedZoom} setZoom={(z) => expandedZoom.set(z)} source={props.source} kind={scene.kind} />
            <IconButton icon="close" label={t("action.close")} onPress={() => expanded.set(false)} />
          </View>
        </View>
        <Scroll style={{ flex: 1, direction: "ltr" }}>
          <Scroll horizontal contentStyle={{ padding: 24, flexGrow: 1, justifyContent: "center" }}>
            <SceneView scene={scene} zoom={expandedZoom()} />
          </Scroll>
        </Scroll>
      </Dialog>
    </View>
  );
}
