import assert from "node:assert/strict";
import { test } from "node:test";
import { liveryCandidates, modelCode, plainLiveries } from "../src/lib/liveries.ts";

test("car names map to the game's model codes", () => {
  assert.equal(modelCode("Ferrari 499P"), "499P");
  assert.equal(modelCode("Porsche 963"), "963");
  assert.equal(modelCode("Porsche 911 GT3 R LMGT3"), "911GT3R");
  assert.equal(modelCode("Aston Martin Vantage AMR LMGT"), "AMV");
  assert.equal(modelCode("Oreca 07 Gibson"), "ORECA07");
  assert.equal(modelCode("Unknown Prototype"), null);
});

test("plain liveries are newest season first, ELMS spec first for ELMS cars", () => {
  assert.deepEqual(plainLiveries("Ferrari 499P").slice(0, 2), ["397_26_499P", "397_25_499P"]);
  assert.equal(plainLiveries("Oreca 07", "LMP2_ELMS")[0], "397_26ELMS_ORECA07");
  assert.equal(plainLiveries("Aston Martin Vantage AMR LMGT", "GT3", "Racing Spirit of Léman 2025 #59:ELMS")[0], "397_26ELMS_AMV");
  assert.equal(plainLiveries("Oreca 07", "LMP2")[0], "397_26_ORECA07");
});

test("the lap's own livery comes before plain ones", () => {
  assert.deepEqual(liveryCandidates("83_26_AFCO45368276", "Ferrari 499P").slice(0, 2), ["83_26_AFCO45368276", "397_26_499P"]);
  assert.equal(liveryCandidates("", "Ferrari 499P")[0], "397_26_499P");
  assert.deepEqual(liveryCandidates(null, "Mystery car"), []);
});
