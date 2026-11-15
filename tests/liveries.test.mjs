import assert from "node:assert/strict";
import { test } from "node:test";
import { liveryCandidates, liveryForVehicle, modelCode, plainLiveries } from "../src/lib/liveries.ts";

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

test("a vehicle name finds its livery among the game's renders", () => {
  const names = [
    "12_26_GCHA18747807",
    "12_26_JOTA50003117",
    "23_26_THOR59931582",
    "23_26_UNIT14272908",
    "36_24_ALPI18C9931",
    "36_26_ALPI20490502",
    "59_25_RSL_DF69FBFB",
    "59_26_RSL_16524330",
    "83_26_AFCO45368276",
    "83_AFCORSE7FBCECA5",
  ];
  assert.equal(liveryForVehicle("Alpine Endurance Team 2026 #36:WEC", names), "36_26_ALPI20490502");
  assert.equal(liveryForVehicle("Alpine Endurance Team 2024 #36:WEC", names), "36_24_ALPI18C9931");
  assert.equal(liveryForVehicle("Cadillac Hertz Team Jota 2026 #12:LM", names), "12_26_JOTA50003117");
  assert.equal(liveryForVehicle("Heart of Racing Team 2026 #23:WEC", names), "23_26_THOR59931582");
  assert.equal(liveryForVehicle("United Autosports 2026 #23:LM", names), "23_26_UNIT14272908");
  assert.equal(liveryForVehicle("Racing Spirit of Léman  2026 #59:LM", names), "59_26_RSL_16524330");
  assert.equal(liveryForVehicle("Racing Spirit of Léman 2025 #59:ELMS", names), "59_25_RSL_DF69FBFB");
  assert.equal(liveryForVehicle("AF Corse 2023 #83:LM", names), "83_AFCORSE7FBCECA5");
  // Another team with the same number, a season the game lacks, or no number at all.
  assert.equal(liveryForVehicle("Iron Lynx 2026 #12:LM", names), null);
  assert.equal(liveryForVehicle("AF Corse 2025 #83:LM", names), null);
  assert.equal(liveryForVehicle("Alpine A424", names), null);
  assert.equal(liveryForVehicle("", names), null);
});

test("a livery found by vehicle name comes after the lap's own and before plain ones", () => {
  const names = ["36_26_ALPI20490502"];
  const alpine = ["Alpine A424", "Hyper", "Alpine Endurance Team 2026 #36:WEC", names];
  assert.deepEqual(liveryCandidates("", ...alpine).slice(0, 2), ["36_26_ALPI20490502", "397_26_ALPINE"]);
  assert.deepEqual(liveryCandidates("36_26_ALPI82796866", ...alpine).slice(0, 3), [
    "36_26_ALPI82796866",
    "36_26_ALPI20490502",
    "397_26_ALPINE",
  ]);
});
