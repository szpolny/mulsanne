# Mulsanne

In-game overlay and telemetry companion for **Le Mans Ultimate**, named after the Mulsanne Straight. The app icon source is `src-tauri/icons/source/app-icon.svg` (regenerate with `pnpm tauri icon src-tauri/icons/source/app-icon.svg`).

- **Overlay HUD**: a transparent, click-through window on top of the game with movable, scalable widgets.
- **Companion app**: records every lap you drive and keeps your history across sessions and personal bests. You can analyse laps with synced telemetry traces and an interactive track map.

Data comes from LMU's built-in shared memory (`LMU_Data`), so no game plugin is needed.

## Widgets

| Widget | Shows |
| --- | --- |
| Dashboard | speed, gear, RPM/shift bar, pedal bars, TC / ABS / map / brake bias |
| Input trace | rolling throttle, brake and steering graph |
| Delta | live delta to your all-time PB (from your own recorded laps) or the session best |
| Lap timing | current, last, session best, PB, predicted, sector splits (PB / session best colours) |
| Standings | multi-class tower with car number, brand logo, tyre compound and tread used (or laps on set), gaps; player always visible |
| Relative | cars around you on track with number, logo, time gaps and lapped/lapping colours |
| Fuel & energy | fuel and virtual energy per lap, laps left, amount to add to finish |
| Tyres | 3-zone surface temps, pressures, wear, brake temps |
| Session | session type, time/laps remaining, position, time of day |
| Flags | FCY, sector yellows, blue flag, start lights, penalties, a short track-limits warning when you pick up a point |
| Track map | outline from your recorded laps, live car positions by class |
| Weather, Damage, Electronics | off by default |

## Usage

1. Run LMU in **borderless** or windowed mode. A transparent overlay can't draw over exclusive fullscreen.
2. Start Mulsanne. The companion window opens and the overlay covers the primary monitor.
3. Press **Ctrl+Shift+F9** (or "Edit overlay layout") to drag widgets and scroll over them to resize. Press it again to finish.
4. **Ctrl+Shift+F10** shows or hides the overlay.
5. Enable widgets and tune their options on the **Overlays** page.
6. Without the game running, turn on **Demo data** on the Live page to design your layout.

Laps are saved automatically to `%APPDATA%\com.lm-overlay.app\laps.db` (SQLite) when you cross the line. A lap counts towards personal bests only when it's valid (not invalidated by track limits) and is neither an out-lap nor an in-lap.

## Development

Stack: Tauri 2 (Rust), Svelte 5, Vite, Tailwind CSS 4, uPlot, SQLite (rusqlite).

```bash
pnpm install
pnpm tauri dev
```

```bash
pnpm check                      # svelte-check / TypeScript
cd src-tauri && cargo test      # layout, recorder and database tests
```

**UI preview in a browser.** Outside Tauri, `pnpm dev` swaps in a mocked backend (`src/dev/mock.ts`) fed by demo fixtures. Generate the fixtures once:

```bash
cd src-tauri && cargo run --example export_fixtures -- ../src/dev/fixtures
```

Then open `http://localhost:1420/` (companion), `/#overlay` or `/#overlay-edit`.

**Checking the game connection.** With LMU on track, run this to print what the app reads, with no windows:

```bash
cd src-tauri && cargo run --example lmu_probe
```

### Layout

```
src-tauri/src/
  lmu/layout.rs   LMU_Data struct mirror (sizes/offsets verified against the SDK with MSVC)
  lmu/shm.rs      mapping, frame events and the game's shared lock
  lmu/mod.rs      raw data -> Snapshot
  demo.rs         simulated car and field
  recorder.rs     sessions, laps, sectors, PB delta, fuel/VE strategy
  db.rs           SQLite storage, gzipped per-lap telemetry
  engine.rs       data loop + lap writer threads, emits `snapshot` events
  commands.rs     Tauri commands
src/
  overlay/        HUD window and widgets
  companion/      companion app pages (Live, Overlays, Sessions, Analysis, Records, Settings)
  lib/            shared stores, types, formatting, analysis maths
```

Car logos come from [Simple Icons](https://simpleicons.org) (CC0). Brands it does not cover show a monogram. Logos are trademarks of their owners.
