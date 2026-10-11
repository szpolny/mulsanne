import { invoke } from "@tauri-apps/api/core";
import type {
  Filters,
  LapRecord,
  LapTrace,
  PersonalBest,
  SessionSummary,
  Snapshot,
  Status,
  TrackMap,
} from "./types";

export const api = {
  getSnapshot: () => invoke<Snapshot>("get_snapshot"),
  getStatus: () => invoke<Status>("get_status"),
  getConfig: () => invoke<unknown>("get_config"),
  saveConfig: (config: unknown) => invoke<void>("save_config", { config }),
  setEditMode: (enabled: boolean) => invoke<void>("set_edit_mode", { enabled }),
  setOverlayVisible: (visible: boolean) => invoke<void>("set_overlay_visible", { visible }),
  setDemoMode: (enabled: boolean) => invoke<void>("set_demo_mode", { enabled }),
  listSessions: (opts: { track?: string | null; car?: string | null; limit?: number; offset?: number } = {}) =>
    invoke<SessionSummary[]>("list_sessions", opts),
  listLaps: (sessionId: number) => invoke<LapRecord[]>("list_laps", { sessionId }),
  listTrackLaps: (track: string, car?: string | null, validOnly = false, limit = 200) =>
    invoke<LapRecord[]>("list_track_laps", { track, car, validOnly, limit }),
  getLapTrace: (lapId: number) => invoke<LapTrace>("get_lap_trace", { lapId }),
  personalBests: () => invoke<PersonalBest[]>("personal_bests"),
  getFilters: () => invoke<Filters>("get_filters"),
  getTrackMap: (track: string) => invoke<TrackMap | null>("get_track_map", { track }),
  deleteLap: (lapId: number) => invoke<void>("delete_lap", { lapId }),
  deleteSession: (sessionId: number) => invoke<void>("delete_session", { sessionId }),
};
