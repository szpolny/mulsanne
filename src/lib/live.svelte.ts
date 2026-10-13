import { listen } from "@tauri-apps/api/event";
import { getContext, setContext } from "svelte";
import { api } from "./api";
import type { Snapshot, Status } from "./types";

class Live {
  /** Latest telemetry snapshot. `$state.raw` avoids deep-proxying ~40 KB of data at 30 Hz. */
  snap = $state.raw<Snapshot | null>(null);
  status = $state.raw<Status>({ demo: false, editMode: false, overlayVisible: true, gameFocused: false, dataDir: "" });
}

export const live = new Live();

/** Where widgets read telemetry from. */
export interface LiveSource {
  readonly snap: Snapshot | null;
}

const LIVE_KEY = Symbol("live");

/** Gives child widgets a different telemetry source (e.g. the companion's preview). */
export function provideLive(source: LiveSource) {
  setContext(LIVE_KEY, source);
}

/** The telemetry source for a widget: the one provided by a parent, else the global live data. */
export function useLive(): LiveSource {
  return getContext<LiveSource | undefined>(LIVE_KEY) ?? live;
}

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
