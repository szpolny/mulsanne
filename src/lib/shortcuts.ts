import { api } from "./api";
import { config } from "./config.svelte";
import { live } from "./live.svelte";

const MODIFIERS = ["Control", "Shift", "Alt", "Meta", "OS"];

// KeyboardEvent.code → key name understood by the global shortcut parser.
function keyName(code: string) {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  return code;
}

/** The shortcut a key press forms, e.g. "Ctrl+Shift+F9"; null for a lone modifier. */
export function accelerator(e: KeyboardEvent): string | null {
  if (MODIFIERS.includes(e.key)) return null;
  const parts = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean);
  return [...parts, keyName(e.code)].join("+");
}

/**
 * Runs the edit and visibility shortcuts while this window has keyboard focus.
 * With a Wayland HUD the global shortcuts only see keys sent to XWayland
 * windows such as the game, not the app's own native windows.
 */
export function handleLocalShortcuts(): () => void {
  function onkeydown(e: KeyboardEvent) {
    if (!live.status.waylandOverlay || e.defaultPrevented || e.repeat) return;
    const pressed = accelerator(e);
    const g = config.value.general;
    if (pressed === g.editShortcut) {
      e.preventDefault();
      void api.setEditMode(!live.status.editMode);
    } else if (pressed === g.visibilityShortcut) {
      e.preventDefault();
      void api.setOverlayVisible(!live.status.overlayVisible);
    }
  }
  window.addEventListener("keydown", onkeydown);
  return () => window.removeEventListener("keydown", onkeydown);
}
