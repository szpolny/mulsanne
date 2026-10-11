import { listen } from "@tauri-apps/api/event";
import { api } from "./api";

export type Units = "metric" | "imperial";

export type OptionDef =
  | { key: string; label: string; type: "number"; min: number; max: number; step: number }
  | { key: string; label: string; type: "boolean" }
  | { key: string; label: string; type: "select"; choices: { value: string; label: string }[] };

export interface WidgetMeta {
  id: string;
  name: string;
  description: string;
  defaults: { enabled: boolean; x: number; y: number };
  options: OptionDef[];
  optionDefaults: Record<string, unknown>;
}

export interface WidgetConfig {
  enabled: boolean;
  x: number;
  y: number;
  scale: number;
  opacity: number;
  options: Record<string, any>;
}

export interface AppConfig {
  version: 1;
  general: {
    units: Units;
    updateHz: number;
    /** Hide widgets while in menus/garage (always shown in edit mode). */
    onlyOnTrack: boolean;
    /** Grid snap in edit mode (px). */
    snap: number;
  };
  widgets: Record<string, WidgetConfig>;
}

export const WIDGETS: WidgetMeta[] = [
  {
    id: "dashboard",
    name: "Dashboard",
    description: "Speed, gear, RPM bar, pedal bars and car settings at a glance.",
    defaults: { enabled: true, x: 760, y: 830 },
    options: [
      { key: "showPedals", label: "Pedal bars", type: "boolean" },
      { key: "showSettings", label: "TC / ABS / map / bias row", type: "boolean" },
    ],
    optionDefaults: { showPedals: true, showSettings: true },
  },
  {
    id: "inputs",
    name: "Input trace",
    description: "Rolling throttle, brake and steering graph.",
    defaults: { enabled: true, x: 1110, y: 960 },
    options: [
      { key: "seconds", label: "History (s)", type: "number", min: 2, max: 20, step: 1 },
      { key: "showSteering", label: "Steering line", type: "boolean" },
      { key: "width", label: "Width (px)", type: "number", min: 200, max: 600, step: 10 },
    ],
    optionDefaults: { seconds: 6, showSteering: true, width: 320 },
  },
  {
    id: "delta",
    name: "Delta",
    description: "Live delta bar against your personal best or the session best.",
    defaults: { enabled: true, x: 810, y: 80 },
    options: [
      {
        key: "reference",
        label: "Reference",
        type: "select",
        choices: [
          { value: "pb", label: "Personal best (all time)" },
          { value: "session", label: "Session best (game)" },
        ],
      },
      { key: "range", label: "Bar range (s)", type: "number", min: 0.5, max: 5, step: 0.5 },
    ],
    optionDefaults: { reference: "pb", range: 2 },
  },
  {
    id: "laptiming",
    name: "Lap timing",
    description: "Current, last, best and predicted lap with sector splits.",
    defaults: { enabled: true, x: 1590, y: 20 },
    options: [{ key: "showSectors", label: "Sectors", type: "boolean" }],
    optionDefaults: { showSectors: true },
  },
  {
    id: "standings",
    name: "Standings",
    description: "Multi-class timing tower.",
    defaults: { enabled: true, x: 20, y: 110 },
    options: [
      { key: "maxRows", label: "Rows per class", type: "number", min: 3, max: 40, step: 1 },
      {
        key: "classes",
        label: "Classes",
        type: "select",
        choices: [
          { value: "all", label: "All classes" },
          { value: "mine", label: "My class only" },
        ],
      },
      { key: "showLogo", label: "Car logo", type: "boolean" },
      { key: "showTyres", label: "Tyres", type: "boolean" },
      {
        key: "tyreInfo",
        label: "Tyre usage as",
        type: "select",
        choices: [
          { value: "wear", label: "Tread used (%)" },
          { value: "laps", label: "Laps on set" },
        ],
      },
      { key: "showLast", label: "Last lap", type: "boolean" },
      { key: "showInterval", label: "Interval", type: "boolean" },
    ],
    optionDefaults: { maxRows: 8, classes: "all", showLogo: true, showTyres: true, tyreInfo: "wear", showLast: true, showInterval: true },
  },
  {
    id: "relative",
    name: "Relative",
    description: "Cars physically around you on track with time gaps.",
    defaults: { enabled: true, x: 1530, y: 560 },
    options: [
      { key: "ahead", label: "Cars ahead", type: "number", min: 1, max: 8, step: 1 },
      { key: "behind", label: "Cars behind", type: "number", min: 1, max: 8, step: 1 },
    ],
    optionDefaults: { ahead: 3, behind: 3 },
  },
  {
    id: "fuel",
    name: "Fuel & energy",
    description: "Fuel and virtual energy per lap, laps left and amount needed to finish.",
    defaults: { enabled: true, x: 1590, y: 300 },
    options: [],
    optionDefaults: {},
  },
  {
    id: "tyres",
    name: "Tyres",
    description: "Tyre temperatures (I/M/O), pressures, wear and brake temps.",
    defaults: { enabled: true, x: 20, y: 820 },
    options: [{ key: "showBrakes", label: "Brake temperatures", type: "boolean" }],
    optionDefaults: { showBrakes: true },
  },
  {
    id: "session",
    name: "Session",
    description: "Session type, time remaining, time of day and lap count.",
    defaults: { enabled: true, x: 20, y: 20 },
    options: [],
    optionDefaults: {},
  },
  {
    id: "flags",
    name: "Flags",
    description: "Race control flags: yellow sectors, FCY, blue flag, start lights.",
    defaults: { enabled: true, x: 860, y: 20 },
    options: [],
    optionDefaults: {},
  },
  {
    id: "trackmap",
    name: "Track map",
    description: "Track outline with live car positions.",
    defaults: { enabled: true, x: 1640, y: 790 },
    options: [
      { key: "size", label: "Size (px)", type: "number", min: 120, max: 500, step: 10 },
      { key: "classColors", label: "Colour by class", type: "boolean" },
    ],
    optionDefaults: { size: 240, classColors: true },
  },
  {
    id: "weather",
    name: "Weather",
    description: "Air/track temperature, rain, wetness and wind.",
    defaults: { enabled: false, x: 1590, y: 190 },
    options: [],
    optionDefaults: {},
  },
  {
    id: "damage",
    name: "Damage",
    description: "Body damage zones, detached parts, overheating and impacts.",
    defaults: { enabled: false, x: 330, y: 860 },
    options: [],
    optionDefaults: {},
  },
  {
    id: "electronics",
    name: "Electronics",
    description: "TC, ABS, motor map, migration, anti-roll bars and hybrid state.",
    defaults: { enabled: false, x: 560, y: 930 },
    options: [],
    optionDefaults: {},
  },
];

export const widgetMeta = (id: string) => WIDGETS.find((w) => w.id === id)!;

function defaults(): AppConfig {
  return {
    version: 1,
    general: { units: "metric", updateHz: 30, onlyOnTrack: true, snap: 4 },
    widgets: Object.fromEntries(
      WIDGETS.map((w) => [
        w.id,
        { ...w.defaults, scale: 1, opacity: 0.92, options: structuredClone(w.optionDefaults) } satisfies WidgetConfig,
      ]),
    ),
  };
}

/** Fills anything missing in a stored config from the defaults. */
function merge(stored: any): AppConfig {
  const base = defaults();
  if (!stored || typeof stored !== "object") return base;
  const out: AppConfig = { ...base, general: { ...base.general, ...(stored.general ?? {}) } };
  for (const id of Object.keys(base.widgets)) {
    const s = stored.widgets?.[id] ?? {};
    out.widgets[id] = {
      ...base.widgets[id],
      ...s,
      options: { ...base.widgets[id].options, ...(s.options ?? {}) },
    };
  }
  return out;
}

class ConfigStore {
  value = $state<AppConfig>(defaults());
  loaded = $state(false);
}

export const config = new ConfigStore();

let saveTimer: ReturnType<typeof setTimeout> | undefined;
let started = false;
let lastSaved = "";

export async function loadConfig() {
  if (started) return;
  started = true;
  await listen<unknown>("config-changed", (e) => {
    // Skip our own save echoing back; it may be older than local edits.
    if (JSON.stringify(e.payload) === lastSaved) return;
    config.value = merge(e.payload);
  });
  config.value = merge(await api.getConfig());
  config.loaded = true;
}

/** Persists the current config (debounced) and broadcasts it to the other window. */
export function saveConfig() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    const snapshot = $state.snapshot(config.value);
    lastSaved = JSON.stringify(snapshot);
    api.saveConfig(snapshot);
  }, 250);
}

export function resetLayout() {
  const d = defaults();
  for (const [id, w] of Object.entries(config.value.widgets)) {
    w.x = d.widgets[id].x;
    w.y = d.widgets[id].y;
    w.scale = 1;
  }
  saveConfig();
}
