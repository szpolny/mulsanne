import { mount } from "svelte";
import "./app.css";
import Overlay from "./overlay/Overlay.svelte";
import Companion from "./companion/Companion.svelte";

// Outside Tauri (plain browser during development) fake the backend. `VITE_DEMO=1` does
// the same inside the app window, to review the UI with demo sessions and rankings.
const inApp = "__TAURI_INTERNALS__" in window;
if (import.meta.env.DEV && (!inApp || (import.meta.env.VITE_DEMO && !location.hash.startsWith("#overlay")))) {
  await (await import("./dev/mock")).install(inApp);
}

// The overlay window loads `index.html#overlay`; everything else is the companion app.
const isOverlay = location.hash.startsWith("#overlay");
document.body.classList.add(isOverlay ? "overlay" : "companion");

const app = mount(isOverlay ? Overlay : Companion, {
  target: document.getElementById("app")!,
});

export default app;
