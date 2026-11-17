import assert from "node:assert/strict";
import { test } from "node:test";
import { classLabel, classOrder, dateLocale, dateTime } from "../src/lib/format.ts";

test("POSIX locales do not prevent the WebKit UI from loading", () => {
  for (const languages of [["C"], ["C.UTF-8"], ["POSIX"], []]) {
    const locale = dateLocale(languages);
    assert.equal(locale, "en-US");
    assert.doesNotThrow(() => new Intl.DateTimeFormat(locale).format(new Date()));
  }
});

test("valid language preferences and regional formatting are preserved", () => {
  assert.equal(dateLocale(["pl-PL", "en-US"]), "pl-PL");
  assert.equal(dateLocale(["pl_PL.UTF-8"]), "pl-PL");
  assert.equal(dateLocale(["C", "de-DE"]), "de-DE");
  assert.ok(dateTime(1700000000));
});

test("series-spec classes read as the class itself", () => {
  assert.equal(classLabel("LMP2_ELMS"), "LMP2");
  assert.equal(classLabel("LMP2"), "LMP2");
  assert.equal(classLabel("Hypercar"), "Hypercar");
  assert.ok(classOrder("Hypercar") < classOrder("LMP2") && classOrder("LMP2") < classOrder("LMP2_ELMS") && classOrder("LMP2_ELMS") < classOrder("GT3"));
});

test("race times show hours only when a race runs past one", async () => {
  const { raceTime } = await import("../src/lib/format.ts");
  assert.equal(raceTime(2533.456), "42:13.456");
  assert.equal(raceTime(3725.1), "1:02:05.100");
  assert.equal(raceTime(3600 + 5.25), "1:00:05.250");
  assert.equal(raceTime(null), "–");
});
