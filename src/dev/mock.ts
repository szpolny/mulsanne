// Browser-only preview: fakes the Tauri backend with demo fixtures so the UI can
// be developed in a normal browser (`pnpm dev`, then open http://localhost:1420
// or http://localhost:1420/#overlay; add ?nolive to simulate the game not running).
// Generate fixtures first:
//   cd src-tauri && cargo run --example export_fixtures -- ../src/dev/fixtures
import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";
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

  mockIPC(
    (cmd, args: any) => {
      switch (cmd) {
        case "plugin:app|version":
          return version;
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
          return fx.sessions;
        case "list_laps":
          return fx.laps;
        case "list_track_laps":
          return [...fx.laps].sort((a: any, b: any) => a.lapTime - b.lapTime);
        case "get_lap_trace":
          return fx.traces[String(args.lapId)];
        case "personal_bests":
          return fx.personalBests;
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
