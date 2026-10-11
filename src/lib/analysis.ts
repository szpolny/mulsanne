import type { LapTrace } from "./types";

/** Channels resampled onto a common distance grid. */
export interface Channels {
  t: Float64Array;
  speed: Float64Array;
  throttle: Float64Array;
  brake: Float64Array;
  steering: Float64Array;
  gear: Float64Array;
  rpm: Float64Array;
  x: Float64Array;
  z: Float64Array;
  gLat: Float64Array;
  gLong: Float64Array;
  abs: Float64Array;
  tc: Float64Array;
  tyreFl: Float64Array;
  tyreFr: Float64Array;
  tyreRl: Float64Array;
  tyreRr: Float64Array;
}

export interface Aligned {
  d: Float64Array;
  a: Channels;
  b: Channels | null;
  /** tA - tB at each distance (positive = A slower). */
  delta: Float64Array | null;
}

const LINEAR: (keyof Channels & keyof LapTrace)[] = [
  "t", "speed", "throttle", "brake", "steering", "rpm", "x", "z", "gLat", "gLong",
  "tyreFl", "tyreFr", "tyreRl", "tyreRr",
];
const STEP: (keyof Channels & keyof LapTrace)[] = ["gear", "abs", "tc"];

/** Indices where distance strictly increases (laps can stall in place). */
function monotonic(d: number[]): number[] {
  const idx: number[] = [];
  let last = -Infinity;
  for (let i = 0; i < d.length; i++) {
    if (d[i] > last) {
      idx.push(i);
      last = d[i];
    }
  }
  return idx;
}

function resample(trace: LapTrace, grid: Float64Array): Channels {
  const idx = monotonic(trace.d);
  const n = grid.length;
  const out = {} as Channels;
  for (const k of [...LINEAR, ...STEP]) out[k] = new Float64Array(n);
  let j = 0;
  for (let g = 0; g < n; g++) {
    const dist = grid[g];
    while (j < idx.length - 2 && trace.d[idx[j + 1]] < dist) j++;
    const i0 = idx[j];
    const i1 = idx[Math.min(j + 1, idx.length - 1)];
    const d0 = trace.d[i0];
    const d1 = trace.d[i1];
    const f = d1 > d0 ? Math.max(0, Math.min(1, (dist - d0) / (d1 - d0))) : 0;
    for (const k of LINEAR) {
      const arr = trace[k] as number[];
      if (!arr?.length) continue;
      out[k][g] = arr[i0] + (arr[i1] - arr[i0]) * f;
    }
    for (const k of STEP) {
      const arr = trace[k] as number[];
      if (!arr?.length) continue;
      out[k][g] = f < 0.5 ? arr[i0] : arr[i1];
    }
  }
  return out;
}

export function align(a: LapTrace, b: LapTrace | null, step = 2): Aligned {
  const endA = a.d.at(-1) ?? 0;
  const end = b ? Math.min(endA, b.d.at(-1) ?? 0) : endA;
  const start = Math.max(a.d[0] ?? 0, b?.d[0] ?? 0);
  const n = Math.max(2, Math.floor((end - start) / step) + 1);
  const d = new Float64Array(n);
  for (let i = 0; i < n; i++) d[i] = start + i * step;

  const ra = resample(a, d);
  const rb = b ? resample(b, d) : null;
  let delta: Float64Array | null = null;
  if (rb) {
    delta = new Float64Array(n);
    for (let i = 0; i < n; i++) delta[i] = ra.t[i] - rb.t[i];
  }
  return { d, a: ra, b: rb, delta };
}

/** Index of the grid point closest to distance `dist`. */
export function indexAt(d: Float64Array, dist: number): number {
  let lo = 0;
  let hi = d.length - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (d[mid] < dist) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

export interface BrakeZone {
  /** Grid index where braking starts. */
  start: number;
  end: number;
  entrySpeed: number;
  minSpeed: number;
  minSpeedIdx: number;
}

/** Braking zones: brake > 15% for at least 15 m, merged when separated by short gaps. */
export function brakeZones(c: Channels, d: Float64Array): BrakeZone[] {
  const zones: BrakeZone[] = [];
  let i = 0;
  const n = d.length;
  while (i < n) {
    if (c.brake[i] <= 0.15) {
      i++;
      continue;
    }
    const start = i;
    let end = i;
    let gap = 0;
    while (i < n && (c.brake[i] > 0.15 || gap < 10)) {
      if (c.brake[i] > 0.15) {
        end = i;
        gap = 0;
      } else gap++;
      i++;
    }
    if (d[end] - d[start] < 15) continue;
    // Minimum speed until throttle is picked up again after the zone.
    let minIdx = end;
    let j = end;
    while (j < n && c.throttle[j] < 0.5) {
      if (c.speed[j] < c.speed[minIdx]) minIdx = j;
      j++;
    }
    for (let k = start; k <= end; k++) if (c.speed[k] < c.speed[minIdx]) minIdx = k;
    zones.push({ start, end, entrySpeed: c.speed[start], minSpeed: c.speed[minIdx], minSpeedIdx: minIdx });
  }
  return zones;
}
