// Small, consistent controls built from NSH primitives and the palette.

import { Text, View } from "nsh";
import { Icon, type IconName } from "./Icon";
import { palette } from "./theme";

export function IconButton(props: {
  icon: IconName;
  label: string;
  onPress: () => void;
  size?: number;
  active?: boolean;
  disabled?: boolean;
  tone?: "default" | "onChrome";
}) {
  const p = palette;
  const box = () => (props.size ?? 18) + 14;
  return (
    <View
      role="button"
      accessibilityLabel={props.label}
      accessibilityState={{ selected: !!props.active, disabled: !!props.disabled }}
      disabled={props.disabled}
      onPress={() => props.onPress()}
      style={{
        width: box(),
        height: box(),
        borderRadius: 8,
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: props.active ? p().selected : "transparent",
        cursor: "pointer",
        opacity: props.disabled ? 0.4 : 1,
      }}
      hoverStyle={{ backgroundColor: props.active ? p().selected : p().hover }}
      activeStyle={{ backgroundColor: p().pressed }}
    >
      <Icon name={props.icon} size={props.size ?? 18} color={props.active ? p().accent : p().textSoft} />
    </View>
  );
}

export function Button(props: {
  label: string;
  onPress: () => void;
  icon?: IconName;
  kind?: "primary" | "secondary" | "ghost" | "danger";
  hint?: string;
  compact?: boolean;
}) {
  const p = palette;
  const kind = () => props.kind ?? "secondary";
  const bg = () => {
    switch (kind()) {
      case "primary":
        return p().accent;
      case "danger":
        return p().dangerSoft;
      case "ghost":
        return "transparent";
      default:
        return p().page;
    }
  };
  const fg = () => (kind() === "primary" ? p().onAccent : kind() === "danger" ? p().danger : p().text);
  return (
    <View
      role="button"
      accessibilityLabel={props.label}
      accessibilityHint={props.hint}
      onPress={() => props.onPress()}
      style={{
        flexDirection: "row",
        alignItems: "center",
        gap: 8,
        paddingHorizontal: props.compact ? 10 : 16,
        height: props.compact ? 30 : 38,
        borderRadius: 9,
        backgroundColor: bg(),
        borderWidth: kind() === "secondary" ? 1 : 0,
        borderColor: p().border,
        cursor: "pointer",
      }}
      hoverStyle={{ backgroundColor: kind() === "primary" ? p().accentStrong : p().hover }}
      activeStyle={{ opacity: 0.85 }}
    >
      {props.icon ? <Icon name={props.icon} size={16} color={fg()} /> : null}
      <Text style={{ color: fg(), fontSize: 14, fontWeight: "600" }}>{props.label}</Text>
      {props.hint ? <Text style={{ color: kind() === "primary" ? p().onAccent : p().textMuted, fontSize: 12, opacity: 0.8 }}>{props.hint}</Text> : null}
    </View>
  );
}

export function Segmented<T extends string>(props: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  label: string;
}) {
  const p = palette;
  return (
    <View
      accessibilityLabel={props.label}
      style={{ flexDirection: "row", backgroundColor: p().chrome, borderRadius: 10, padding: 3, gap: 3, alignSelf: "flex-start" }}
    >
      {props.options.map((o) => (
        <View
          role="button"
          accessibilityLabel={o.label}
          accessibilityState={{ selected: props.value === o.value }}
          onPress={() => props.onChange(o.value)}
          style={{
            paddingHorizontal: 14,
            height: 30,
            borderRadius: 8,
            justifyContent: "center",
            backgroundColor: props.value === o.value ? p().page : "transparent",
            borderWidth: props.value === o.value ? 1 : 0,
            borderColor: p().border,
            cursor: "pointer",
          }}
          hoverStyle={{ backgroundColor: props.value === o.value ? p().page : p().hover }}
        >
          <Text style={{ fontSize: 13, fontWeight: props.value === o.value ? "600" : "400", color: props.value === o.value ? p().text : p().textSoft }}>
            {o.label}
          </Text>
        </View>
      ))}
    </View>
  );
}

export function Stepper(props: { value: string; onDecrease: () => void; onIncrease: () => void; label: string }) {
  const p = palette;
  return (
    <View accessibilityLabel={props.label} style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
      <IconButton icon="minus" label={`${props.label} −`} onPress={props.onDecrease} size={16} />
      <View style={{ minWidth: 56, height: 30, borderRadius: 8, backgroundColor: p().chrome, alignItems: "center", justifyContent: "center", paddingHorizontal: 8 }}>
        <Text style={{ fontSize: 13, fontWeight: "600", color: p().text }}>{props.value}</Text>
      </View>
      <IconButton icon="plus" label={`${props.label} +`} onPress={props.onIncrease} size={16} />
    </View>
  );
}

export function Toggle(props: { value: boolean; onChange: (v: boolean) => void; label: string }) {
  const p = palette;
  return (
    <View
      role="switch"
      accessibilityLabel={props.label}
      accessibilityState={{ checked: props.value }}
      onPress={() => props.onChange(!props.value)}
      style={{
        width: 42,
        height: 24,
        borderRadius: 12,
        padding: 3,
        backgroundColor: props.value ? p().accent : p().pressed,
        alignItems: props.value ? "flex-end" : "flex-start",
        cursor: "pointer",
        direction: "ltr",
      }}
    >
      <View style={{ width: 18, height: 18, borderRadius: 9, backgroundColor: "#FFFFFF" }} />
    </View>
  );
}

export function Kbd(props: { keys: string }) {
  const p = palette;
  return (
    <View style={{ direction: "ltr", paddingHorizontal: 7, height: 22, borderRadius: 6, borderWidth: 1, borderColor: p().border, backgroundColor: p().page, justifyContent: "center" }}>
      <Text style={{ fontSize: 12, color: p().textSoft, fontFamily: "Segoe UI" }}>{props.keys}</Text>
    </View>
  );
}
