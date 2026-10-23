// Browser-only preview: fakes the Tauri backend with demo fixtures so the UI can
// be developed in a normal browser (`pnpm dev`, then open http://localhost:1420
// or http://localhost:1420/#overlay; add ?nolive to simulate the game not running).
// Generate fixtures first:
//   cd src-tauri && cargo run --example export_fixtures -- ../src/dev/fixtures
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { version } from "../../package.json";

export async function install() {
  const res = await fetch(new URL("./fixtures/fixtures.json", import.meta.url));
  if (!res.ok) {
    console.warn("dev mock: fixtures missing, run the export_fixtures example");
    return;
  }
  const fx = await res.json();
  let config: unknown = null;
  let editMode = location.hash.includes("edit");
  const noLive = new URLSearchParams(location.search).has("nolive");
  let previewFrame = 0;
  let onlineMe: { id: string; displayName: string | null; createdAt: number } | null = null;
  const uploadStates = new Map<number, { lapId: number; status: string; remoteId: string | null; error: string | null; trace: string | null }>();

  mockWindows("main");
  mockIPC(
    (cmd, args: any) => {
      switch (cmd) {
        case "plugin:app|version":
          return version;
        case "plugin:opener|open_url":
          window.open(args.url, "_blank", "noopener,noreferrer");
          return;
        case "get_snapshot":
          return noLive ? null : fx.frames[0];
        case "get_preview_snapshot":
          return fx.frames[previewFrame++ % fx.frames.length];
        case "get_status":
          return { demo: true, editMode, overlayVisible: true, gameFocused: false, dataDir: "C:\\mock" };
        case "get_config":
          return config;
        case "save_config":
          config = args.config;
          emit("config-changed", config);
          return;
        case "set_edit_mode":
          editMode = args.enabled;
          emit("edit-mode", editMode);
          return;
        case "list_sessions":
          // Fixtures come from the demo sim; present them as game data so online uploads can be previewed.
          return fx.sessions.map((s: any) => ({ ...s, source: "lmu", gameVersion: 1124 }));
        case "list_laps":
          return fx.laps;
        case "list_track_laps":
          return [...fx.laps].sort((a: any, b: any) => a.lapTime - b.lapTime);
        case "get_lap_trace":
          return fx.traces[String(args.lapId)];
        case "personal_bests":
          return fx.personalBests.map((b: any) => ({ ...b, fromGame: true }));
        case "get_filters":
          return fx.filters;
        case "get_track_map":
          return fx.trackMap;
        case "daily_races": {
          const at = (min: number) => Math.ceil(Date.now() / 300_000) * 300_000 + min * 60_000;
          const race = (name: string, tier: string, track: string, duration: string, first: number, every: number) => ({
            name, tier, track, duration, image: null,
            starts: [0, 1, 2, 3].map((i) => at(first + i * every)).filter((t) => t < at(0) + 4 * 3_600_000),
          });
          return [
            race("Logitech McLaren G Challenge Winter - Q2", "beginner", "Monza", "30 minutes", 5, 45),
            race("LMP3 Fixed", "beginner", "Le Mans", "32 minutes", 15, 45),
            race("LMGT3 Fixed", "beginner", "Fuji Wec 2023", "32 minutes", 25, 45),
            race("ELMS Sprint Trophy", "intermediate", "Spa Francorchamps", "44 minutes", 10, 60),
            race("LMGT3 Sprint Cup", "intermediate", "Laguna Seca", "44 minutes", 30, 60),
            race("Prototype Fixed", "intermediate", "Long Beach 2026", "44 minutes", 40, 60),
            race("One Stint Sprint", "advanced", "Bahrain Wec 2023", "55 minutes", 20, 90),
            race("ELMS Super 60", "advanced", "Portimao Wec 2023", "1h 13 minutes", 50, 90),
            race("WEC-Xperience", "advanced", "Silverstone 2025", "1h 14 minutes", 70, 90),
          ];
        }
        case "online_register":
          onlineMe ??= { id: "mock-user", displayName: args.displayName ?? null, createdAt: Math.floor(Date.now() / 1000) };
          return onlineMe;
        case "online_me":
          if (!onlineMe) throw "Online features are not set up on this PC";
          return onlineMe;
        case "online_set_display_name":
          if (!onlineMe) throw "Online features are not set up on this PC";
          onlineMe.displayName = args.displayName?.trim() || null;
          return onlineMe;
        case "online_delete_data":
          onlineMe = null;
          uploadStates.clear();
          emit("lap-uploads-changed", null);
          return;
        case "upload_laps": {
          const ids: number[] = args.lapIds.filter((id: number) => !uploadStates.has(id));
          for (const id of ids) uploadStates.set(id, { lapId: id, status: "queued", remoteId: null, error: null, trace: null });
          emit("lap-uploads-changed", null);
          setTimeout(() => {
            for (const id of ids) uploadStates.set(id, { lapId: id, status: "ranked", remoteId: `mock-${id}`, error: null, trace: null });
            emit("lap-uploads-changed", null);
          }, 800);
          return ids.length;
        }
        case "ranked_lap_trace": {
          // Another driver's lap: a fixture lap made ~2% faster.
          const t = Object.values(fx.traces)[1] as any;
          return { ...t, t: t.t.map((x: number) => x * 0.98), speed: t.speed.map((x: number) => x * 1.02) };
        }
        case "lap_upload_states":
          return [...uploadStates.values()];
        case "remove_lap_upload":
          uploadStates.delete(args.lapId);
          emit("lap-uploads-changed", null);
          return;
        case "leaderboard_boards": {
          const pbs: any[] = fx.personalBests;
          return pbs.map((b) => ({ track: b.track, carClass: b.carClass, drivers: 12, bestLap: b.lapTime - 1.234, gameVersions: [1124, 1123] }));
        }
        case "leaderboard": {
          const q = args.query;
          const pb = (fx.personalBests as any[]).find((b) => b.track === q.track && b.carClass === q.carClass);
          const names = ["A. Laurent", null, "K. Tanaka", "J. Smith", null, "L. Novak", "P. Silva", "D. Müller"];
          const entries = names.map((driver, i) => {
            const t = (pb?.lapTime ?? 100) - 1.234 + i * 0.41;
            return {
              rank: i + 1, lapId: `other-${i}`, driver, car: pb?.car ?? "Car", vehicleName: "", sessionType: i % 2 ? "Race" : "Practice",
              gameVersion: i < 5 ? 1124 : 1123,
              hasTrace: i !== 2,
              lapTime: t, s1: t * 0.3, s2: t * 0.4, s3: t * 0.3, recordedAt: Math.floor(Date.now() / 1000) - i * 86400,
            };
          });
          const mine = pb && uploadStates.get(pb.lapId)?.remoteId;
          if (mine) {
            entries.splice(3, 0, { ...entries[3], lapId: mine, driver: onlineMe?.displayName ?? null, lapTime: pb.lapTime, s1: pb.s1, s2: pb.s2, s3: pb.s3 });
            entries.forEach((e, i) => (e.rank = i + 1));
          }
          const filtered = entries.filter(
            (e) => (!q.sessionType || e.sessionType === q.sessionType) && (!q.gameVersion || e.gameVersion === q.gameVersion),
          );
          return { entries: filtered, total: filtered.length };
        }
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );

  if (noLive) return;
  let i = 0;
  setInterval(() => {
    emit("snapshot", fx.frames[i++ % fx.frames.length]);
  }, 50);
}
