<script lang="ts">
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import { classColor, classLabel, classOrder, dateTime, lapTime, sectorTime } from "../../lib/format";
  import CarRender from "../../lib/CarRender.svelte";
  import CarSilhouette from "../../lib/CarSilhouette.svelte";
  import { liveryCandidates } from "../../lib/liveries";
  import type { BoardSummary, LeaderboardEntry } from "../../lib/types";
  import TrackOutline from "../../lib/TrackOutline.svelte";
  import { uploads } from "../../lib/uploads.svelte";
  import { nav } from "../nav.svelte";

  const PAGE = 50;
  const SESSION_TYPES = ["Practice", "Qualifying", "Race", "Warmup", "Test Day"];
  const PODIUM = ["#e6b422", "#b8bcc6", "#c8834a"];

  interface TrackGroup {
    track: string;
    /** Fastest class first. */
    boards: BoardSummary[];
    drivers: number;
  }

  const enabled = $derived(config.value.online.enabled);
  let boards = $state<BoardSummary[]>([]);
  let board = $state<BoardSummary | null>(null);
  let search = $state("");
  let sessionType = $state("");
  /** "" for all versions. */
  let gameVersion = $state("");
  let entries = $state<LeaderboardEntry[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state("");

  /** Server ids of laps uploaded from this PC, to highlight them. */
  const mine = $derived(new Set(Object.values(uploads.byLap).map((u) => u.remoteId).filter(Boolean)));

  /** One entry per track (the server sends one board per track and class), sorted by name. */
  const tracks = $derived.by(() => {
    const m = new Map<string, TrackGroup>();
    for (const b of boards) {
      const g = m.get(b.track) ?? { track: b.track, boards: [], drivers: 0 };
      g.boards.push(b);
      g.drivers += b.drivers;
      m.set(b.track, g);
    }
    const out = [...m.values()];
    for (const g of out) g.boards.sort((a, b) => classOrder(a.carClass) - classOrder(b.carClass) || a.carClass.localeCompare(b.carClass));
    return out.sort((a, b) => a.track.localeCompare(b.track, undefined, { sensitivity: "base" }));
  });

  const shown = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return q ? tracks.filter((g) => g.track.toLowerCase().includes(q)) : tracks;
  });

  const group = $derived(board ? (tracks.find((g) => g.track === board!.track) ?? null) : null);

  /** The fastest three, shown large above the table. Ties can put several on one step. */
  const podium = $derived(entries.filter((e) => e.rank <= 3).slice(0, 3));

  /** The lap's livery, then plain ones of its model for laps without one or with a livery this install lacks. */
  const liveries = (e: LeaderboardEntry) => liveryCandidates(e.vehFile, e.car, board?.carClass, e.vehicleName);

  /** "AF Corse 2026 #83:LM" -> "AF Corse 2026 #83"; nothing when it only repeats the car. */
  function teamName(e: LeaderboardEntry): string {
    const name = e.vehicleName.replace(/:[A-Z0-9]+$/i, "").trim();
    return name && name !== e.car ? name : "";
  }

  /** "LMP2"; only when a track ranks both LMP2 specs is the ELMS one called out. */
  function tabLabel(b: BoardSummary, siblings: BoardSummary[]): string {
    const label = classLabel(b.carClass);
    const clash = siblings.some((s) => s !== b && classLabel(s.carClass) === label);
    return clash && b.carClass !== label ? b.carClass.replace(/_/g, " ") : label;
  }

  function selectTrack(g: TrackGroup) {
    // Keep the class when switching tracks, if the new track ranks it.
    board = g.boards.find((b) => b.carClass === board?.carClass) ?? g.boards[0];
  }

  async function loadBoards() {
    error = "";
    try {
      boards = await api.leaderboardBoards();
      const same = board && boards.find((b) => b.track === board!.track && b.carClass === board!.carClass);
      board = same ?? (tracks[0]?.boards[0] ?? null);
    } catch (e) {
      error = String(e);
    }
  }

  async function loadEntries(more = false) {
    if (!board) {
      entries = [];
      total = 0;
      return;
    }
    loading = true;
    error = "";
    try {
      const res = await api.leaderboard({
        track: board.track,
        carClass: board.carClass,
        sessionType: sessionType || null,
        gameVersion: gameVersion ? Number(gameVersion) : null,
        limit: PAGE,
        offset: more ? entries.length : 0,
      });
      entries = more ? [...entries, ...res.entries] : res.entries;
      total = res.total;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (enabled) void loadBoards();
  });

  $effect(() => {
    void board;
    void sessionType;
    void gameVersion;
    if (enabled) void loadEntries();
  });

  // A version filter only makes sense on boards that have laps from it.
  $effect(() => {
    if (gameVersion && !board?.gameVersions.includes(Number(gameVersion))) gameVersion = "";
  });
</script>

{#if !enabled}
  <div class="mx-auto max-w-3xl p-6">
    <h1 class="mb-5 page-title">Rankings</h1>
    <div class="card p-10 text-center">
      <p class="mb-4 text-muted">Compare your lap times with other drivers. Turn on online features to see the rankings.</p>
      <button class="btn btn-primary" onclick={() => (nav.page = "settings")}>Open Settings</button>
    </div>
  </div>
{:else}
  <div class="flex h-full">
    <aside class="flex w-[280px] shrink-0 flex-col border-r border-line">
      <div class="flex items-center justify-between px-4 pt-4 pb-2">
        <span class="section-title">Tracks <span class="font-normal text-faint">{tracks.length || ""}</span></span>
        <button class="rounded px-2 py-0.5 text-xs text-accent hover:bg-surface-3" onclick={loadBoards}>Refresh</button>
      </div>
      {#if tracks.length > 6}
        <div class="px-3 pb-2">
          <input class="field w-full text-sm" type="search" placeholder="Find a track" bind:value={search} aria-label="Find a track" />
        </div>
      {/if}
      <div class="flex-1 overflow-y-auto px-2 pb-2">
        {#each shown as g (g.track)}
          {@const active = board?.track === g.track}
          <button
            class={["mb-0.5 flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left", active ? "bg-surface-3" : "hover:bg-surface-2"]}
            onclick={() => selectTrack(g)}
          >
            <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded bg-surface-2">
              <TrackOutline track={g.track} hideMissing class="h-8 w-8 {active ? 'text-accent' : 'text-fg/70'}" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm font-medium">{g.track}</span>
              <span class="mt-1 flex items-center gap-1">
                {#each g.boards as b (b.carClass)}
                  <span class="h-1.5 w-4 rounded-full" style:background={classColor(b.carClass)} title={tabLabel(b, g.boards)}></span>
                {/each}
                <span class="ml-1 text-[11px] text-muted">{g.drivers} {g.drivers === 1 ? "lap" : "laps"}</span>
              </span>
            </span>
          </button>
        {:else}
          <div class="p-6 text-center text-sm text-muted">
            {#if error}{:else if search}No track matches “{search}”.{:else}No ranked laps yet. Upload yours from Records.{/if}
          </div>
        {/each}
      </div>
    </aside>

    <section class="min-w-0 flex-1 overflow-y-auto">
      {#if board && group}
        <div class="relative overflow-hidden border-b border-line bg-gradient-to-r from-surface-2 to-transparent px-6 pt-5">
          <TrackOutline track={board.track} hideMissing class="pointer-events-none absolute top-3 right-6 h-[calc(100%-1.5rem)] w-40 text-fg/15" />
          <div class="relative">
            <h1 class="page-title">{board.track}</h1>
            <p class="mt-0.5 text-sm text-muted">Best lap per driver</p>
          </div>
          <div class="relative mt-4 flex flex-wrap gap-1" role="tablist" aria-label="Car class">
            {#each group.boards as b (b.carClass)}
              {@const active = b.carClass === board.carClass}
              <button
                role="tab"
                aria-selected={active}
                class={["flex items-center gap-2 rounded-t-md border-x border-t px-3.5 py-2 text-sm", active ? "border-line bg-surface font-semibold" : "border-transparent text-muted hover:text-fg"]}
                style:box-shadow={active ? `inset 0 3px 0 ${classColor(b.carClass)}` : undefined}
                onclick={() => (board = b)}
              >
                <span class="h-2 w-2 rounded-full" style:background={classColor(b.carClass)}></span>
                {tabLabel(b, group.boards)}
                <span class="text-xs font-normal text-muted tnum">{lapTime(b.bestLap)}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="p-6">
          {#if error}
            <p class="mb-4 text-sm text-bad">{error}</p>
          {/if}
          <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
            <div class="text-sm text-muted">
              {board.drivers}
              {board.drivers === 1 ? "driver" : "drivers"} in {classLabel(board.carClass)}
            </div>
            <div class="flex gap-2">
              <select class="field text-sm" bind:value={sessionType} aria-label="Session type">
                <option value="">All sessions</option>
                {#each SESSION_TYPES as t}<option value={t}>{t}</option>{/each}
              </select>
              {#if board.gameVersions.length}
                <select class="field text-sm" bind:value={gameVersion} aria-label="Game version">
                  <option value="">All game versions</option>
                  {#each board.gameVersions as v}<option value={String(v)}>Game version {v}</option>{/each}
                </select>
              {/if}
            </div>
          </div>

          {#if podium.length}
            <!-- 2nd, 1st, 3rd, like a real podium. -->
            <div class="mb-4 grid grid-cols-3 items-end gap-3">
              {#each [podium[1], podium[0], podium[2]] as e, i (e?.lapId ?? `empty-${i}`)}
                {#if e}
                  {@const color = PODIUM[e.rank - 1] ?? PODIUM[2]}
                  {@const first = e.rank === 1}
                  <div
                    class={["card relative overflow-hidden px-4 pb-3 text-center", first ? "pt-3" : "pt-2", mine.has(e.lapId) && "ring-1 ring-accent"]}
                    style:background-image="linear-gradient(to bottom, {color}33, transparent 70%)"
                    style:box-shadow="inset 0 3px 0 {color}"
                  >
                    <span
                      class="absolute top-3 left-3 inline-flex h-7 w-7 items-center justify-center rounded-full text-sm font-bold text-black"
                      style:background={color}>{e.rank}</span
                    >
                    <div class={["flex items-center justify-center", first ? "h-32" : "h-24"]}>
                      <CarRender
                        file={liveries(e)}
                        view="front"
                        alt={e.car}
                        class="h-full w-full object-contain drop-shadow-[0_8px_12px_rgb(0_0_0/0.5)]"
                      >
                        {#snippet fallback()}<CarSilhouette class="h-2/3 w-2/3 text-muted" />{/snippet}
                      </CarRender>
                    </div>
                    <div class="mt-1 truncate font-semibold">
                      {#if e.driver}{e.driver}{:else}<span class="font-normal text-muted">Anonymous</span>{/if}
                    </div>
                    <div class="truncate text-xs text-muted">{e.car}{teamName(e) ? ` · ${teamName(e)}` : ""}</div>
                    <div class={["mt-1.5 font-bold tnum", first ? "text-2xl text-best" : "text-lg"]}>{lapTime(e.lapTime)}</div>
                    <div class="text-xs text-muted tnum">{first ? "Fastest lap" : `+${(e.lapTime - podium[0].lapTime).toFixed(3)}`}</div>
                  </div>
                {:else}
                  <div></div>
                {/if}
              {/each}
            </div>
          {/if}

          <div class="card overflow-x-auto">
            <table class="w-full text-sm whitespace-nowrap tnum">
              <thead class="bg-surface-2 text-left text-xs text-muted">
                <tr>
                  <th class="w-14 px-3 py-2 font-medium">#</th>
                  <th class="px-3 py-2 font-medium">Driver</th>
                  <th class="px-3 py-2 font-medium">Car</th>
                  <th class="px-3 py-2 text-right font-medium">Time</th>
                  <th class="px-3 py-2 text-right font-medium">Gap</th>
                  <th class="hidden px-3 py-2 font-medium 2xl:table-cell">Sectors</th>
                  <th class="px-3 py-2 font-medium">Session</th>
                  <th class="px-3 py-2 font-medium"></th>
                </tr>
              </thead>
              <tbody>
                {#each entries as e (e.lapId)}
                  {@const you = mine.has(e.lapId)}
                  <tr class={["border-t border-line", you ? "bg-accent/15" : "hover:bg-surface-2"]}>
                    <td class="px-3 py-2">
                      {#if e.rank <= 3}
                        <span
                          class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[11px] font-bold text-black"
                          style:background={PODIUM[e.rank - 1]}>{e.rank}</span
                        >
                      {:else}
                        <span class="inline-block w-5 text-center font-semibold text-muted">{e.rank}</span>
                      {/if}
                    </td>
                    <td class="px-3 py-2 font-medium">
                      {#if e.driver}{e.driver}{:else}<span class="font-normal text-muted">Anonymous</span>{/if}
                      {#if you}<span class="ml-1 rounded bg-accent px-1 text-[10px] font-bold text-white">YOU</span>{/if}
                    </td>
                    <td class="px-3 py-1">
                      <div class="flex items-center gap-3">
                        <!-- Fixed box so rows line up while the image loads. -->
                        <span class="flex h-9 w-[84px] shrink-0 items-center justify-center">
                          <CarRender file={liveries(e)} view="thumb" alt={e.car} class="h-9 w-[84px] object-contain">
                            {#snippet fallback()}<CarSilhouette class="h-7 w-[72px] text-muted" />{/snippet}
                          </CarRender>
                        </span>
                        <span class="min-w-0">
                          <span class="block">{e.car}</span>
                          {#if teamName(e)}<span class="block text-xs text-muted">{teamName(e)}</span>{/if}
                        </span>
                      </div>
                    </td>
                    <td class={["px-3 py-2 text-right font-semibold", e.rank === 1 && "text-best"]}>{lapTime(e.lapTime)}</td>
                    <td class="px-3 py-2 text-right text-muted">{e.rank === 1 ? "–" : `+${(e.lapTime - entries[0].lapTime).toFixed(3)}`}</td>
                    <td class="hidden px-3 py-2 text-muted 2xl:table-cell">{sectorTime(e.s1)} · {sectorTime(e.s2)} · {sectorTime(e.s3)}</td>
                    <td class="px-3 py-1 text-muted">
                      <span class="block">{e.sessionType}</span>
                      <span class="block text-xs">{dateTime(e.recordedAt)}</span>
                    </td>
                    <td class="px-2 py-2 text-right">
                      {#if e.hasTrace && !you}
                        <button
                          class="rounded px-2 py-0.5 text-xs text-accent hover:bg-surface-3"
                          title="Compare your best lap on this track with this one"
                          onclick={() =>
                            nav.compareWithRanked(board!.track, {
                              id: e.lapId,
                              label: `${e.driver ?? "Anonymous"} · ${e.car}`,
                              lapTime: e.lapTime,
                              s1: e.s1,
                              s2: e.s2,
                              s3: e.s3,
                            })}>Compare</button
                        >
                      {/if}
                    </td>
                  </tr>
                {:else}
                  <tr><td colspan="8" class="px-3 py-8 text-center text-muted">{loading ? "Loading…" : "No ranked laps for this filter."}</td></tr>
                {/each}
              </tbody>
            </table>
          </div>
          {#if entries.length < total}
            <div class="mt-3 text-center">
              <button class="btn" disabled={loading} onclick={() => loadEntries(true)}>Show more ({total - entries.length})</button>
            </div>
          {/if}
        </div>
      {:else if error}
        <p class="p-6 text-sm text-bad">{error}</p>
      {/if}
    </section>
  </div>
{/if}
