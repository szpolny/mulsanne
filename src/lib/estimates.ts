import type { Strategy, Vehicle } from "./types";

/** Laps until the stint ends: the first of fuel, energy or the session running out. Null before there is usage data. */
export function stintLapsLeft(st: Pick<Strategy, "lapsOfFuel" | "lapsOfVe" | "lapsRemaining">): number | null {
  const usage = [st.lapsOfFuel, st.lapsOfVe].filter((v) => v > 0);
  if (!usage.length) return null;
  return Math.min(...usage, ...(st.lapsRemaining > 0 ? [st.lapsRemaining] : []));
}

/** Tread left (0..1) after `laps` more laps at `perLap` wear. */
export function wearAtStintEnd(wear: number, perLap: number, laps: number): number {
  return Math.max(0, wear - perLap * laps);
}

type Timed = Pick<Vehicle, "lapDist" | "timeIntoLap" | "estimatedLapTime">;

/** Fraction of the lap in time terms, null when the game has no usable estimate for this car. */
function timeProgress(car: Timed): number | null {
  if (car.estimatedLapTime <= 0 || car.timeIntoLap <= 0) return null;
  const progress = car.timeIntoLap / car.estimatedLapTime;
  return progress <= 1.2 ? progress : null;
}

/** Shortest way round the lap, -0.5..0.5. */
const wrapLap = (laps: number) => laps - Math.round(laps);

/**
 * Seconds from `me` to `other` on track, positive when `other` is ahead.
 *
 * Time into the lap is used because cars cover ground at very different speeds around a lap.
 * Each car is scaled by its own estimated lap time so classes with different pace stay comparable.
 * Without those estimates it falls back to distance at `fallbackLapTime`.
 */
export function relativeGap(me: Timed, other: Timed, trackLength: number, fallbackLapTime: number): number {
  const mine = timeProgress(me);
  const theirs = timeProgress(other);
  if (mine != null && theirs != null) return wrapLap(theirs - mine) * me.estimatedLapTime;
  return wrapLap((other.lapDist - me.lapDist) / (trackLength || 1)) * fallbackLapTime;
}
