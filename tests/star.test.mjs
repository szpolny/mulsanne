import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

// Exercise the actual prompt controller with a fake clock, config, and browser.
const source = readFileSync(new URL("../src/lib/star.svelte.ts", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
});

function app(stored = {}) {
  let now = 0;
  let timer;
  let listener;
  let saved;
  const general = { starPromptUsageMs: 0, starPromptDone: false, ...stored };
  const context = {
    exports: {},
    $state: (value) => value,
    config: { value: { general } },
    saveConfig: () => { saved = { ...general }; },
    openUrl: async () => {},
    performance: { now: () => now },
    document: {
      visibilityState: "visible",
      addEventListener: (_, fn) => { listener = fn; },
      removeEventListener: () => { listener = undefined; },
    },
    setInterval: (fn) => { timer = fn; return 1; },
    clearInterval: () => { timer = undefined; },
  };
  vm.runInNewContext(outputText, context);
  context.exports.initStarPrompt();
  return {
    ...context.exports,
    general,
    saved: () => saved,
    advance(ms) { now += ms; timer?.(); },
    visible(value) {
      context.document.visibilityState = value ? "visible" : "hidden";
      listener?.();
    },
  };
}

test("launch counts cannot trigger the prompt; visible usage persists across runs", () => {
  let current = app({ launches: 100 });
  assert.equal(current.starPrompt.open, false);
  for (let i = 0; i < 60; i++) current.advance(15_000);
  assert.equal(current.starPrompt.open, false);
  assert.equal(current.saved().starPromptUsageMs, 15 * 60_000);
  current = app(current.saved());
  assert.equal(current.starPrompt.open, false);
  for (let i = 0; i < 59; i++) current.advance(15_000);
  assert.equal(current.starPrompt.open, false);
  current.advance(15_000);
  assert.equal(current.starPrompt.open, true);
});

test("hidden time and long suspension do not satisfy the usage threshold", () => {
  const current = app();
  current.visible(false);
  current.advance(60 * 60_000);
  current.visible(true);
  assert.equal(current.general.starPromptUsageMs, 0);
  current.advance(60 * 60_000);
  assert.equal(current.general.starPromptUsageMs, 15_000);
  assert.equal(current.starPrompt.open, false);
});

test("dismissal and starring permanently retire the prompt", async () => {
  for (const action of ["dismissStarPrompt", "starOnGitHub"]) {
    const current = app({ starPromptUsageMs: 30 * 60_000 });
    current.advance(15_000);
    assert.equal(current.starPrompt.open, true);
    await current[action]();
    assert.equal(current.starPrompt.open, false);
    const restarted = app(current.saved());
    restarted.advance(60 * 60_000);
    assert.equal(restarted.starPrompt.open, false);
    assert.equal(restarted.general.starPromptDone, true);
  }
});
