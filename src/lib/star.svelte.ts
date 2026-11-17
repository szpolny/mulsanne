import { openUrl } from "@tauri-apps/plugin-opener";
import { config, saveConfig } from "./config.svelte";

export const REPO_URL = "https://github.com/szpolny/mulsanne";

/** Give people half an hour with the companion before asking. */
const USAGE_BEFORE_PROMPT_MS = 30 * 60 * 1000;
const TICK_MS = 15 * 1000;

export const starPrompt = $state({ open: false });

let stopTracking: (() => void) | undefined;

/** Accumulates visible companion time across runs; hidden or suspended time doesn't count. */
export function initStarPrompt() {
  if (stopTracking) return;
  const general = config.value.general;
  if (general.starPromptDone) return;
  let lastTick = performance.now();
  let visible = document.visibilityState === "visible";
  const tick = () => {
    const now = performance.now();
    const elapsed = Math.min(TICK_MS, Math.max(0, now - lastTick));
    lastTick = now;
    const general = config.value.general;
    if (general.starPromptDone) {
      stopTracking?.();
      return;
    }
    if (visible && elapsed > 0) {
      general.starPromptUsageMs += elapsed;
      saveConfig();
    }
    visible = document.visibilityState === "visible";
    if (visible && general.starPromptUsageMs >= USAGE_BEFORE_PROMPT_MS) {
      starPrompt.open = true;
      stopTracking?.();
    }
  };
  const timer = setInterval(tick, TICK_MS);
  document.addEventListener("visibilitychange", tick);
  stopTracking = () => {
    clearInterval(timer);
    document.removeEventListener("visibilitychange", tick);
    stopTracking = undefined;
  };
}

function markStarPromptDone() {
  stopTracking?.();
  config.value.general.starPromptDone = true;
  saveConfig();
}

/** Closes the popup for good. */
export function dismissStarPrompt() {
  starPrompt.open = false;
  markStarPromptDone();
}

/** Opens the repository page and retires the popup, even if the browser fails to open. */
export async function starOnGitHub() {
  markStarPromptDone();
  await openUrl(REPO_URL);
  starPrompt.open = false;
}
