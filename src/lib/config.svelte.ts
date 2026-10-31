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
  /** Size multiplier for this widget, on top of the global scale. */
  size: number;
  /** Opacity for this widget, on top of the global opacity. */
  alpha: number;
  options: Record<string, any>;
}

export interface AppConfig {
  version: 1;
  general: {
    units: Units;
    updateHz: number;
    /** Hide widgets while in menus/garage (always shown in edit mode). */
    onlyOnTrack: boolean;
    /** Hide widgets while the game window is not focused (always shown in edit/demo mode). */
    hideUnfocused: boolean;
    /** Show release notes once after each app update. */
    showChangelog: boolean;
    /** Last installed version whose release notes were handled. */
    lastSeenVersion: string;
    /** Grid snap in edit mode (px). */
    snap: number;
    /** Size multiplier applied to every widget. */
    scale: number;
    /** Opacity applied to every widget. */
    opacity: number;
    /** Name of the monitor the overlay covers; "" for the primary monitor. */
    monitor: string;
  };
  /** Lap rankings and car setup sharing. Off until the user opts in. */
  online: {
    /** Master switch; while off the app makes no requests to the Mulsanne server. */
    enabled: boolean;
    /** Upload laps only when asked, or every eligible lap as it's recorded. */
    lapUpload: "manual" | "auto";
    /** Hide your display name on laps uploaded from now on. */
    anonymous: boolean;
    /** Send the telemetry of your best lap on each track and class. */
    uploadTraces: boolean;
  };
  widgets: Record<string, WidgetConfig>;
  /** Id of the profile mirrored by `general.scale/opacity` and `widgets`. */
  activeProfile: string;
  /** Saved overlay setups: layout, widget options, scale and opacity. */
  profiles: Record<string, Profile>;
}

/** A saved overlay setup. App-wide preferences (units, refresh rate, …) are not part of it. */
export interface Profile {
  name: string;
  scale: number;
  opacity: number;
  widgets: Record<string, WidgetConfig>;
}

export const WIDGETS: WidgetMeta[] = [
  {
    id: "dashboard",
    name: "Dashboard",
    description: "Speed, gear, RPM bar, status lights, hybrid battery and car settings at a glance.",
    defaults: { enabled: true, x: 760, y: 830 },
    options: [
      { key: "showIndicators", label: "Status lights (headlights, limiter, TC, ABS, wipers, engine)", type: "boolean" },
      { key: "showBattery", label: "Hybrid battery bar", type: "boolean" },
      { key: "showFfb", label: "FFB output meter", type: "boolean" },
      { key: "showSettings", label: "TC / ABS / map / bias row", type: "boolean" },
    ],
    optionDefaults: { showIndicators: true, showBattery: true, showFfb: false, showSettings: true },
  },
  {
    id: "inputs",
    name: "Input trace",
    description: "Rolling throttle, brake and steering graph with live pedal bars.",
    defaults: { enabled: true, x: 1110, y: 960 },
    options: [
      { key: "seconds", label: "History (s)", type: "number", min: 2, max: 20, step: 1 },
      { key: "showSteering", label: "Steering line", type: "boolean" },
      { key: "showPedals", label: "Throttle / brake / clutch bars", type: "boolean" },
      { key: "showClutch", label: "Clutch bar", type: "boolean" },
      { key: "width", label: "Width (px)", type: "number", min: 200, max: 600, step: 10 },
    ],
    optionDefaults: { seconds: 6, showSteering: true, showPedals: true, showClutch: true, width: 320 },
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
    description: "Multi-class timing tower with optional session header.",
    defaults: { enabled: true, x: 20, y: 110 },
    options: [
      { key: "showSession", label: "Session info header", type: "boolean" },
      { key: "maxRows", label: "Rows for my class", type: "number", min: 3, max: 40, step: 1 },
      { key: "maxRowsOther", label: "Rows for other classes", type: "number", min: 1, max: 40, step: 1 },
      { key: "around", label: "Cars around me outside the top rows", type: "number", min: 0, max: 5, step: 1 },
      {
        key: "classes",
        label: "Classes",
        type: "select",
        choices: [
          { value: "all", label: "All classes" },
          { value: "mine", label: "My class only" },
        ],
      },
      { key: "myClassFirst", label: "My class first", type: "boolean" },
      { key: "showLogo", label: "Car logo", type: "boolean" },
      { key: "showTyres", label: "Tyre compound", type: "boolean" },
      { key: "showEnergy", label: "Virtual energy left", type: "boolean" },
      { key: "showPits", label: "Pit stops", type: "boolean" },
      { key: "showLast", label: "Last lap", type: "boolean" },
      { key: "showBest", label: "Best lap", type: "boolean" },
      { key: "colorTimes", label: "Colour lap times (green = personal best, purple = class best)", type: "boolean" },
      { key: "showInterval", label: "Interval", type: "boolean" },
    ],
    optionDefaults: {
      showSession: false,
      maxRows: 8,
      maxRowsOther: 8,
      around: 1,
      classes: "all",
      myClassFirst: false,
      showLogo: true,
      showTyres: true,
      showEnergy: true,
      showPits: false,
      showLast: true,
      showBest: false,
      colorTimes: true,
      showInterval: true,
    },
  },
  {
    id: "relative",
    name: "Relative",
    description: "Cars physically around you on track with time gaps, virtual energy left and optional lap times.",
    defaults: { enabled: true, x: 1530, y: 560 },
    options: [
      { key: "ahead", label: "Cars ahead", type: "number", min: 1, max: 8, step: 1 },
      { key: "behind", label: "Cars behind", type: "number", min: 1, max: 8, step: 1 },
      { key: "showEnergy", label: "Virtual energy left", type: "boolean" },
      { key: "showLast", label: "Last lap", type: "boolean" },
      { key: "showBest", label: "Best lap", type: "boolean" },
      { key: "colorTimes", label: "Colour lap times (green = personal best, purple = class best)", type: "boolean" },
    ],
    optionDefaults: { ahead: 3, behind: 3, showEnergy: true, showLast: false, showBest: false, colorTimes: true },
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
    description: "Bodywork, aero and suspension damage, detached parts and overheating.",
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

function defaultWidgets(): Record<string, WidgetConfig> {
  return Object.fromEntries(
    WIDGETS.map((w) => [
      w.id,
      { ...w.defaults, size: 1, alpha: 1, options: structuredClone(w.optionDefaults) } satisfies WidgetConfig,
    ]),
  );
}

function defaults(): AppConfig {
  const widgets = defaultWidgets();
  return {
    version: 1,
    general: { units: "metric", updateHz: 30, onlyOnTrack: true, hideUnfocused: false, showChangelog: true, lastSeenVersion: "", snap: 4, scale: 1, opacity: 0.92, monitor: "" },
    online: { enabled: false, lapUpload: "manual", anonymous: false, uploadTraces: true },
    widgets,
    activeProfile: "default",
    profiles: { default: { name: "Default", scale: 1, opacity: 0.92, widgets: structuredClone(widgets) } },
  };
}

/** Fills anything missing in stored widget settings from the defaults. */
function mergeWidgets(stored: any): Record<string, WidgetConfig> {
  const base = defaultWidgets();
  for (const id of Object.keys(base)) {
    // Drop legacy per-widget scale/opacity; size and alpha replace them, on top of the global values.
    const { scale: _scale, opacity: _opacity, ...s } = stored?.[id] ?? {};
    base[id] = { ...base[id], ...s, options: { ...base[id].options, ...(s.options ?? {}) } };
  }
  return base;
}

/** Fills anything missing in a stored config from the defaults. */
function merge(stored: any): AppConfig {
  const base = defaults();
  if (!stored || typeof stored !== "object") return base;
  const general = { ...base.general, ...(stored.general ?? {}) };
  const online = { ...base.online, ...(stored.online ?? {}) };
  const widgets = mergeWidgets(stored.widgets);
  const profiles: Record<string, Profile> = {};
  for (const [id, p] of Object.entries<any>(stored.profiles ?? {})) {
    if (!p || typeof p !== "object") continue;
    profiles[id] = {
      name: typeof p.name === "string" && p.name ? p.name : "Profile",
      scale: typeof p.scale === "number" ? p.scale : base.general.scale,
      opacity: typeof p.opacity === "number" ? p.opacity : base.general.opacity,
      widgets: mergeWidgets(p.widgets),
    };
  }
  let activeProfile = typeof stored.activeProfile === "string" ? stored.activeProfile : "default";
  // Configs from before profiles existed: the current setup becomes the first profile.
  if (!profiles[activeProfile]) {
    activeProfile = Object.keys(profiles)[0] ?? "default";
    profiles[activeProfile] ??= {
      name: "Default",
      scale: general.scale,
      opacity: general.opacity,
      widgets: structuredClone(widgets),
    };
  }
  return { ...base, general, online, widgets, activeProfile, profiles };
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

/** Copies the live layout into the active profile's saved slot. */
function syncActiveProfile() {
  const c = config.value;
  c.profiles[c.activeProfile] = {
    name: c.profiles[c.activeProfile]?.name ?? "Profile",
    scale: c.general.scale,
    opacity: c.general.opacity,
    widgets: $state.snapshot(c.widgets),
  };
}

/** Persists the current config (debounced) and broadcasts it to the other window. */
export function saveConfig() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    syncActiveProfile();
    const snapshot = $state.snapshot(config.value);
    lastSaved = JSON.stringify(snapshot);
    api.saveConfig(snapshot);
  }, 250);
}

export function resetLayout() {
  const d = defaultWidgets();
  for (const [id, w] of Object.entries(config.value.widgets)) {
    w.x = d[id].x;
    w.y = d[id].y;
    w.size = 1;
  }
  config.value.general.scale = 1;
  saveConfig();
}

/** Makes `id` the active profile and loads its layout into the overlay. */
export function switchProfile(id: string) {
  const c = config.value;
  const p = c.profiles[id];
  if (!p || id === c.activeProfile) return;
  syncActiveProfile();
  c.activeProfile = id;
  c.general.scale = p.scale;
  c.general.opacity = p.opacity;
  c.widgets = mergeWidgets($state.snapshot(p.widgets));
  saveConfig();
}

/** Creates a profile from the current setup (or the defaults) and switches to it. */
export function createProfile(name: string, fromDefaults = false) {
  syncActiveProfile();
  const c = config.value;
  const id = `p${Date.now().toString(36)}`;
  const d = defaults();
  c.profiles[id] = fromDefaults
    ? { name, scale: d.general.scale, opacity: d.general.opacity, widgets: d.widgets }
    : { name, scale: c.general.scale, opacity: c.general.opacity, widgets: $state.snapshot(c.widgets) };
  switchProfile(id);
}

export function renameProfile(id: string, name: string) {
  const p = config.value.profiles[id];
  if (!p || !name.trim()) return;
  p.name = name.trim();
  saveConfig();
}

/** Deletes a profile; the last remaining one can't be deleted. */
export function deleteProfile(id: string) {
  const c = config.value;
  const ids = Object.keys(c.profiles);
  if (ids.length < 2 || !c.profiles[id]) return;
  if (id === c.activeProfile) switchProfile(ids.find((x) => x !== id)!);
  delete c.profiles[id];
  saveConfig();
}
