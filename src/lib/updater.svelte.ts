import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

export type UpdatePhase = "idle" | "checking" | "available" | "downloading" | "upToDate" | "error";

export const updater = $state({
  version: "",
  phase: "idle" as UpdatePhase,
  /** Version offered by the latest GitHub release. */
  available: "",
  notes: "",
  /** Download progress, 0..1 (0 while the total size is unknown). */
  progress: 0,
  error: "",
});

let pending: Update | null = null;

const inTauri = () => "__TAURI_INTERNALS__" in window;

export async function checkForUpdate() {
  if (!inTauri() || updater.phase === "checking" || updater.phase === "downloading") return;
  updater.version ||= await getVersion();
  updater.phase = "checking";
  updater.error = "";
  try {
    pending = await check();
    if (pending) {
      updater.available = pending.version;
      updater.notes = pending.body ?? "";
      updater.phase = "available";
    } else {
      updater.phase = "upToDate";
    }
  } catch (e) {
    updater.error = String(e);
    updater.phase = "error";
  }
}

/** Downloads and runs the installer. On Windows the installer closes the app itself. */
export async function installUpdate() {
  if (!pending || updater.phase === "downloading") return;
  updater.phase = "downloading";
  updater.progress = 0;
  let total = 0;
  let received = 0;
  try {
    await pending.downloadAndInstall((event) => {
      if (event.event === "Started") total = event.data.contentLength ?? 0;
      else if (event.event === "Progress") {
        received += event.data.chunkLength;
        if (total) updater.progress = received / total;
      }
    });
    await relaunch();
  } catch (e) {
    updater.error = String(e);
    updater.phase = "error";
  }
}

export async function initUpdater() {
  if (!inTauri()) return;
  updater.version = await getVersion();
  await checkForUpdate();
}
