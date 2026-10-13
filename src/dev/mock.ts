// Browser-only preview: fakes the Tauri backend with demo fixtures so the UI can
// be developed in a normal browser (`pnpm dev`, then open http://localhost:1420
// or http://localhost:1420/#overlay). Generate fixtures first:
//   cd src-tauri && cargo run --example export_fixtures -- ../src/dev/fixtures
import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";

export async function install() {
  const res = await fetch(new URL("./fixtures/fixtures.json", import.meta.url));
  if (!res.ok) {
    console.warn("dev mock: fixtures missing, run the export_fixtures example");
    return;
  }
  const fx = await res.json();
  let config: unknown = null;
  let editMode = location.hash.includes("edit");

  mockIPC(
    (cmd, args: any) => {
      switch (cmd) {
        case "get_snapshot":
          return fx.frames[0];
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
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );

  let i = 0;
  setInterval(() => {
    emit("snapshot", fx.frames[i++ % fx.frames.length]);
  }, 50);
}
