import assert from "node:assert/strict";
import { test } from "node:test";
import { relativeGap, stintLapsLeft, wearAtStintEnd } from "../src/lib/estimates.ts";

const car = (lapDist, timeIntoLap, estimatedLapTime) => ({ lapDist, timeIntoLap, estimatedLapTime });

test("the stint ends with whichever of fuel, energy or the session runs out first", () => {
  assert.equal(stintLapsLeft({ lapsOfFuel: 12, lapsOfVe: 9, lapsRemaining: 30 }), 9);
  assert.equal(stintLapsLeft({ lapsOfFuel: 12, lapsOfVe: 0, lapsRemaining: 30 }), 12);
  assert.equal(stintLapsLeft({ lapsOfFuel: 12, lapsOfVe: 9, lapsRemaining: 4 }), 4);
  assert.equal(stintLapsLeft({ lapsOfFuel: 12, lapsOfVe: 9, lapsRemaining: 0 }), 9);
});

test("no stint estimate before fuel or energy usage is known", () => {
  assert.equal(stintLapsLeft({ lapsOfFuel: 0, lapsOfVe: 0, lapsRemaining: 30 }), null);
});

test("predicted tread stops at zero", () => {
  assert.ok(Math.abs(wearAtStintEnd(0.9, 0.02, 10) - 0.7) < 1e-9);
  assert.equal(wearAtStintEnd(0.1, 0.02, 10), 0);
});

test("relative gaps follow the time into the lap, not the distance", () => {
  // 25% of the distance ahead, but only 10% of the lap time: a fast straight.
  const gap = relativeGap(car(1000, 50, 200), car(2000, 70, 200), 4000, 200);
  assert.ok(Math.abs(gap - 20) < 1e-9);
});

test("relative gaps wrap across the start line", () => {
  const ahead = relativeGap(car(3900, 195, 200), car(100, 5, 200), 4000, 200);
  assert.ok(Math.abs(ahead - 10) < 1e-9);
  const behind = relativeGap(car(100, 5, 200), car(3900, 195, 200), 4000, 200);
  assert.ok(Math.abs(behind + 10) < 1e-9);
});

test("cars of different pace at the same place on track have no gap", () => {
  assert.ok(Math.abs(relativeGap(car(2000, 100, 200), car(2000, 125, 250), 4000, 200)) < 1e-9);
});

test("relative gaps fall back to distance without usable estimates", () => {
  const expected = 50;
  for (const other of [car(2000, 0, 200), car(2000, 100, 0), car(2000, 400, 200)]) {
    assert.ok(Math.abs(relativeGap(car(1000, 50, 200), other, 4000, 200) - expected) < 1e-9);
  }
});
