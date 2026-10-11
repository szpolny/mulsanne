import { mount } from "svelte";
import "./app.css";
import Overlay from "./overlay/Overlay.svelte";
import Companion from "./companion/Companion.svelte";

// Outside Tauri (plain browser during development) fake the backend.
if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
  await (await import("./dev/mock")).install();
}

// The overlay window loads `index.html#overlay`; everything else is the companion app.
const isOverlay = location.hash.startsWith("#overlay");
document.body.classList.add(isOverlay ? "overlay" : "companion");

const app = mount(isOverlay ? Overlay : Companion, {
  target: document.getElementById("app")!,
});

export default app;
