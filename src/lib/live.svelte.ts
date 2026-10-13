import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type { Snapshot, Status } from "./types";

class Live {
  /** Latest telemetry snapshot. `$state.raw` avoids deep-proxying ~40 KB of data at 30 Hz. */
  snap = $state.raw<Snapshot | null>(null);
  status = $state.raw<Status>({ demo: false, editMode: false, overlayVisible: true, gameFocused: false, dataDir: "" });
}

export const live = new Live();

let started = false;

export async function startLive() {
  if (started) return;
  started = true;
  await listen<Snapshot>("snapshot", (e) => {
    live.snap = e.payload;
  });
  await listen<boolean>("edit-mode", (e) => {
    live.status = { ...live.status, editMode: e.payload };
  });
  await listen<boolean>("overlay-visible", (e) => {
    live.status = { ...live.status, overlayVisible: e.payload };
  });
  await listen<boolean>("game-focused", (e) => {
    live.status = { ...live.status, gameFocused: e.payload };
  });
  await listen<boolean>("demo-mode", (e) => {
    live.status = { ...live.status, demo: e.payload };
  });
  const [snap, status] = await Promise.all([api.getSnapshot(), api.getStatus()]);
  live.snap ??= snap;
  live.status = status;
}
