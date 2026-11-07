import assert from "node:assert/strict";
import { test } from "node:test";

test("chart imports survive WebKitGTK's POSIX navigator language", async () => {
  // uPlot initializes its number formatter during import, before a chart exists.
  globalThis.window = { addEventListener() {}, dispatchEvent() {} };
  globalThis.document = {};
  globalThis.devicePixelRatio = 1;
  globalThis.matchMedia = () => ({ addEventListener() {} });
  Object.defineProperty(globalThis, "navigator", { value: { language: "C" }, configurable: true });
  for (const entry of ["uplot/dist/uPlot.esm.js", "uplot"]) {
    const { default: chart } = await import(entry);
    assert.equal(chart.fmtNum(1234.5), "1,234.5");
  }
});
