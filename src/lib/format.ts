import type { Units } from "./config.svelte";

/** 83.456 -> "1:23.456". Non-positive values render as placeholder. */
export function lapTime(s: number | null | undefined, digits = 3): string {
  if (s == null || !isFinite(s) || s <= 0) return "-:--.---";
  const m = Math.floor(s / 60);
  const rest = s - m * 60;
  const sec = rest.toFixed(digits).padStart(digits + 3, "0");
  return m > 0 ? `${m}:${sec}` : rest.toFixed(digits);
}

export function sectorTime(s: number | null | undefined): string {
  if (s == null || !isFinite(s) || s <= 0) return "--.---";
  return lapTime(s);
}

/** Signed delta, e.g. "+0.231" / "-1.04". */
export function delta(s: number | null | undefined, digits = 3): string {
  if (s == null || !isFinite(s)) return "--";
  const sign = s > 0 ? "+" : s < 0 ? "−" : "±";
  return sign + Math.abs(s).toFixed(digits);
}

/** Session clock: 3725 -> "1:02:05". */
export function clock(s: number): string {
  if (!isFinite(s) || s < 0) s = 0;
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = Math.floor(s % 60);
  const mm = String(m).padStart(2, "0");
  const ss = String(sec).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${m}:${ss}`;
}

/** Seconds since midnight -> "14:05". */
export function timeOfDay(s: number): string {
  const t = ((s % 86400) + 86400) % 86400;
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

export function gap(s: number, laps = 0): string {
  if (laps > 0) return `+${laps}L`;
  if (!isFinite(s) || s <= 0) return "--";
  return s >= 60 ? `+${lapTime(s, 1)}` : `+${s.toFixed(1)}`;
}

export function speed(ms: number, units: Units): number {
  return units === "imperial" ? ms * 2.236936 : ms * 3.6;
}
export const speedUnit = (u: Units) => (u === "imperial" ? "mph" : "km/h");

export function temp(c: number, units: Units): number {
  return units === "imperial" ? c * 1.8 + 32 : c;
}
export const tempUnit = (u: Units) => (u === "imperial" ? "°F" : "°C");

export function pressure(kpa: number, units: Units): number {
  return units === "imperial" ? kpa * 0.1450377 : kpa;
}
export const pressureUnit = (u: Units) => (u === "imperial" ? "psi" : "kPa");

export function fuel(l: number, units: Units): number {
  return units === "imperial" ? l * 0.264172 : l;
}
export const fuelUnit = (u: Units) => (u === "imperial" ? "gal" : "L");

/** Virtual energy arrives as a 0..1 fraction; show as percent. */
export function vePct(v: number): number {
  return v <= 1.0001 ? v * 100 : v;
}

export function sessionLabel(kind: string): string {
  return (
    {
      testDay: "Test Day",
      practice: "Practice",
      qualifying: "Qualifying",
      warmup: "Warmup",
      race: "Race",
    } as Record<string, string>
  )[kind] ?? "Session";
}

export function dateTime(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Short driver label: "Jan Kowalski" -> "J. Kowalski". */
export function shortName(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length < 2 || parts[0].endsWith(".")) return name;
  return `${parts[0][0]}. ${parts.slice(1).join(" ")}`;
}

/** Stable colour per class (LMU names vary between "Hyper", "Hypercar", "LMGT3" ...). */
export function classColor(cls: string): string {
  const c = cls.toLowerCase();
  if (c.includes("hyper") || c.includes("lmh") || c.includes("lmdh")) return "#e5484d";
  if (c.includes("lmp2")) return "#3e8ef7";
  if (c.includes("lmp3")) return "#a15ef0";
  if (c.includes("gte")) return "#30a46c";
  if (c.includes("gt3")) return "#f2a33a";
  return "#8b8d98";
}
