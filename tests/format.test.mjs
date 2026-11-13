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
