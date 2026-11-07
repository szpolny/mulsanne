import assert from "node:assert/strict";
import { test } from "node:test";
import { dateLocale, dateTime } from "../src/lib/format.ts";

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
