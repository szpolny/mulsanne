import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type { UploadState } from "./types";

/** Online upload state of local laps, keyed by lap id. */
class Uploads {
  byLap = $state<Record<number, UploadState>>({});
}

export const uploads = new Uploads();

let started = false;

async function refresh() {
  const states = await api.lapUploadStates();
  uploads.byLap = Object.fromEntries(states.map((s) => [s.lapId, s]));
}

/** Loads upload states and keeps them current. Safe to call repeatedly. */
export function watchUploads() {
  if (started) return;
  started = true;
  void refresh();
  void listen("lap-uploads-changed", () => void refresh());
}

/** Flying laps with all three sectors; demo laps are filtered by the backend too. */
export function uploadable(lap: { valid: boolean; outLap?: boolean; inLap?: boolean; s1: number | null; s2: number | null; s3: number | null }) {
  return lap.valid && !lap.outLap && !lap.inLap && lap.s1 != null && lap.s2 != null && lap.s3 != null;
}
