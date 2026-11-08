import { openUrl } from "@tauri-apps/plugin-opener";
import { config, saveConfig } from "./config.svelte";

export const REPO_URL = "https://github.com/szpolny/mulsanne";

/** Give people a few sessions with the app before asking. */
const LAUNCHES_BEFORE_PROMPT = 3;

export const starPrompt = $state({ open: false });

/** Counts this launch and opens the popup once the user has had time to try the app. */
export function initStarPrompt() {
  const general = config.value.general;
  if (general.starPromptDone) return;
  general.launches += 1;
  saveConfig();
  starPrompt.open = general.launches >= LAUNCHES_BEFORE_PROMPT;
}

function markStarPromptDone() {
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
