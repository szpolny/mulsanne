import { WIDGETS, type WidgetMeta } from "../lib/config.svelte";

const GROUPS: { name: string; ids: string[] }[] = [
  { name: "Driving", ids: ["dashboard", "inputs", "electronics"] },
  { name: "Timing", ids: ["laptiming", "delta", "session"] },
  { name: "Race", ids: ["standings", "relative", "flags"] },
  { name: "Car", ids: ["fuel", "tyres", "damage"] },
  { name: "Track", ids: ["trackmap", "weather"] },
];

const grouped = new Set(GROUPS.flatMap((g) => g.ids));

/** Overlay widgets sorted into categories; anything not listed lands in "Other". */
export const WIDGET_GROUPS: { name: string; widgets: WidgetMeta[] }[] = [
  ...GROUPS.map((g) => ({
    name: g.name,
    widgets: g.ids.map((id) => WIDGETS.find((w) => w.id === id)).filter((w): w is WidgetMeta => !!w),
  })),
  { name: "Other", widgets: WIDGETS.filter((w) => !grouped.has(w.id)) },
].filter((g) => g.widgets.length);
